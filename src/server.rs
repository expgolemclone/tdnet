use std::collections::{HashSet, VecDeque};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::extract::{Path, Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::assets::{STYLE_CSS, VIEWER_HTML, VIEWER_JS};
use crate::cli::Period;
use crate::config::{CACHE_MAX_SIZE, TDNET_BASE_URL, TDNET_EXPECTED_HOST};
use crate::disclosure::Disclosure;
use crate::http_client::TdnetClient;
use crate::tdnet_scraper::fetch_disclosures;

#[derive(Clone)]
pub struct AppState {
    client: TdnetClient,
    cache: Arc<Mutex<DateCache>>,
    ticker_filter: Option<HashSet<String>>,
    exclude_tickers: HashSet<String>,
    init_date: String,
    init_period: Period,
}

impl AppState {
    pub fn new(
        client: TdnetClient,
        init_date: String,
        init_period: Period,
        ticker_filter: Option<HashSet<String>>,
        exclude_tickers: HashSet<String>,
    ) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(DateCache::new(CACHE_MAX_SIZE))),
            ticker_filter,
            exclude_tickers,
            init_date,
            init_period,
        }
    }

    pub fn cache_set(&self, date_key: String, items: Vec<Disclosure>) {
        self.cache
            .lock()
            .expect("cache poisoned")
            .set(date_key, items);
    }

    pub fn apply_filter(&self, items: &[Disclosure]) -> Vec<Disclosure> {
        apply_filter(items, self.ticker_filter.as_ref(), &self.exclude_tickers)
    }
}

#[derive(Debug)]
pub struct DateCache {
    max_size: usize,
    items: VecDeque<(String, Vec<Disclosure>)>,
}

impl DateCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            items: VecDeque::new(),
        }
    }

    pub fn set(&mut self, date_key: String, items: Vec<Disclosure>) {
        self.items.retain(|(key, _)| key != &date_key);
        self.items.push_back((date_key, items));
        while self.items.len() > self.max_size {
            self.items.pop_front();
        }
    }

    pub fn get(&mut self, date_key: &str) -> Option<Vec<Disclosure>> {
        let pos = self.items.iter().position(|(key, _)| key == date_key)?;
        let (key, value) = self.items.remove(pos)?;
        let cloned = value.clone();
        self.items.push_back((key, value));
        Some(cloned)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    date: Option<String>,
    period: Option<String>,
}

#[derive(Debug, Serialize)]
struct InitResponse {
    date: String,
    period: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/static/style.css", get(style))
        .route("/static/viewer.js", get(script))
        .route("/api/init", get(api_init))
        .route("/api/list", get(api_list))
        .route("/api/pdf/{filename}", get(proxy_pdf))
        .with_state(state)
}

pub async fn serve(state: AppState, port: u16) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index() -> Html<&'static str> {
    Html(VIEWER_HTML)
}

async fn style() -> impl IntoResponse {
    (
        [
            (CONTENT_TYPE, "text/css; charset=utf-8"),
            (CACHE_CONTROL, "no-store"),
        ],
        STYLE_CSS,
    )
}

async fn script() -> impl IntoResponse {
    (
        [
            (CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (CACHE_CONTROL, "no-store"),
        ],
        VIEWER_JS,
    )
}

async fn api_init(State(state): State<AppState>) -> Json<InitResponse> {
    Json(InitResponse {
        date: state.init_date,
        period: state.init_period.as_str().to_string(),
    })
}

async fn api_list(State(state): State<AppState>, Query(query): Query<ListQuery>) -> Response {
    let Some(end_date) = query.date else {
        return bad_request("date param required (yyyymmdd)");
    };
    if end_date.len() != 8 {
        return bad_request("date param required (yyyymmdd)");
    }
    let period = match query
        .period
        .as_deref()
        .map(Period::parse)
        .unwrap_or(Some(Period::Day))
    {
        Some(period) => period,
        None => {
            return bad_request(format!(
                "invalid period (choose from {:?})",
                Period::valid_values()
            ));
        }
    };
    let dates = match date_range(&end_date, period) {
        Ok(dates) => dates,
        Err(_) => return bad_request("date param required (yyyymmdd)"),
    };

    let mut all_items = Vec::new();
    for date in dates {
        let cached = {
            let mut cache = state.cache.lock().expect("cache poisoned");
            cache.get(&date)
        };
        if let Some(cached) = cached {
            all_items.extend(cached);
            continue;
        }
        match fetch_disclosures(&state.client, &date).await {
            Ok(items) => {
                state.cache_set(date, items.clone());
                all_items.extend(items);
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: err.to_string(),
                    }),
                )
                    .into_response();
            }
        }
    }

    Json(state.apply_filter(&all_items)).into_response()
}

async fn proxy_pdf(State(state): State<AppState>, Path(filename): Path<String>) -> Response {
    if !is_safe_pdf_name(&filename) {
        return not_found();
    }

    let url = format!("{TDNET_BASE_URL}{filename}");
    let Ok(parsed) = reqwest::Url::parse(&url) else {
        return not_found();
    };
    if parsed.host_str() != Some(TDNET_EXPECTED_HOST) {
        return not_found();
    }

    let Ok(resp) = state.client.get_bytes_checked(&url, 30).await else {
        return not_found();
    };

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );
    (headers, resp).into_response()
}

fn bad_request(error: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: error.into(),
        }),
    )
        .into_response()
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not found").into_response()
}

pub fn date_range(end_date_str: &str, period: Period) -> Result<Vec<String>> {
    let end = NaiveDate::parse_from_str(end_date_str, "%Y%m%d")
        .with_context(|| format!("invalid date {end_date_str}"))?;
    let target = period.target_days();
    let mut result = Vec::with_capacity(target);
    let mut offset = 0i64;
    while result.len() < target {
        let date = end - Duration::days(offset);
        if date.weekday().num_days_from_monday() < 5 {
            result.push(date.format("%Y%m%d").to_string());
        }
        offset += 1;
    }
    Ok(result)
}

pub fn apply_filter(
    items: &[Disclosure],
    ticker_filter: Option<&HashSet<String>>,
    exclude_tickers: &HashSet<String>,
) -> Vec<Disclosure> {
    if ticker_filter.is_none() && exclude_tickers.is_empty() {
        return items.to_vec();
    }
    items
        .iter()
        .filter(|item| {
            let ticker = item.code.chars().take(4).collect::<String>();
            ticker_filter.is_none_or(|filter| filter.contains(&ticker))
                && !exclude_tickers.contains(&ticker)
        })
        .cloned()
        .collect()
}

pub fn is_safe_pdf_name(filename: &str) -> bool {
    let Some(stem) = filename.strip_suffix(".pdf") else {
        return false;
    };
    !stem.is_empty()
        && stem
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{DateCache, apply_filter, date_range, is_safe_pdf_name};
    use crate::cli::Period;
    use crate::disclosure::Disclosure;

    #[test]
    fn single_day_range() {
        assert_eq!(date_range("20260325", Period::Day).unwrap(), ["20260325"]);
    }

    #[test]
    fn week_range_skips_weekends_and_descends() {
        let result = date_range("20260327", Period::Week).unwrap();
        assert_eq!(result.len(), 5);
        assert_eq!(
            result,
            ["20260327", "20260326", "20260325", "20260324", "20260323"]
        );
    }

    #[test]
    fn cache_evicts_oldest() {
        let mut cache = DateCache::new(2);
        cache.set("20260101".to_string(), Vec::new());
        cache.set("20260102".to_string(), Vec::new());
        cache.set("20260103".to_string(), Vec::new());
        assert_eq!(cache.len(), 2);
        assert!(cache.get("20260101").is_none());
        assert!(cache.get("20260102").is_some());
    }

    #[test]
    fn filters_tickers() {
        let items = vec![
            Disclosure {
                code: "12340".to_string(),
                pdf: "a.pdf".to_string(),
                ..Disclosure::default()
            },
            Disclosure {
                code: "56780".to_string(),
                pdf: "b.pdf".to_string(),
                ..Disclosure::default()
            },
        ];
        let ticker_filter = HashSet::from(["1234".to_string()]);
        let result = apply_filter(&items, Some(&ticker_filter), &HashSet::new());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "12340");

        let exclude = HashSet::from(["5678".to_string()]);
        let result = apply_filter(&items, None, &exclude);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "12340");
    }

    #[test]
    fn validates_pdf_filename() {
        assert!(is_safe_pdf_name("I_test-001.pdf"));
        assert!(!is_safe_pdf_name("../I_test.pdf"));
        assert!(!is_safe_pdf_name("I_test.txt"));
        assert!(!is_safe_pdf_name(".pdf"));
    }
}
