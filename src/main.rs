use std::collections::HashSet;

use anyhow::Result;
use clap::Parser;
use tdnet_viewer::cli::{
    Args, kill_listeners, load_tickers_from_csv, open_browser, today_yyyymmdd,
};
use tdnet_viewer::config::{DEFAULT_PORT, segments_path};
use tdnet_viewer::http_client::TdnetClient;
use tdnet_viewer::segments::{Segments, fetch_jpx_segments, load_default_segments, save_segments};
use tdnet_viewer::server::{AppState, date_range, serve};
use tdnet_viewer::tdnet_scraper::fetch_disclosures;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let date = args.date.clone().unwrap_or_else(today_yyyymmdd);
    let client = TdnetClient::new()?;

    if args.fetch_segments {
        println!("Fetching segment data from JPX...");
        let data = fetch_jpx_segments(&client).await?;
        let path = segments_path();
        save_segments(&data, &path)?;
        println!("Saved {} entries to {}", data.len(), path.display());
        return Ok(());
    }

    let segments = load_default_segments()?;
    let ticker_filter = build_ticker_filter(&args, &segments)?;
    if let Some(filter) = ticker_filter.as_ref() {
        println!("Filter active: {} tickers", filter.len());
    }

    let mut exclude_tickers = HashSet::new();
    if !args.all && !segments.is_empty() {
        exclude_tickers = segments
            .iter()
            .filter_map(|(code, segment)| {
                if segment.contains("ETF") {
                    Some(code.clone())
                } else {
                    None
                }
            })
            .collect();
        if !exclude_tickers.is_empty() {
            println!("Excluding {} ETF/ETN tickers", exclude_tickers.len());
        }
    }

    let state = AppState::new(
        client.clone(),
        date.clone(),
        args.period,
        ticker_filter,
        exclude_tickers,
    );

    let dates = date_range(&date, args.period)?;
    let mut total = 0usize;
    for (idx, date_key) in dates.iter().enumerate() {
        print!("\rFetching disclosures... ({}/{})", idx + 1, dates.len());
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let items = fetch_disclosures(&client, date_key).await?;
        total += items.len();
        state.cache_set(date_key.clone(), items);
    }
    println!("\rFetched {total} items over {} day(s).     ", dates.len());

    let port = if args.port == 0 {
        DEFAULT_PORT
    } else {
        args.port
    };
    kill_listeners(port);

    let url = format!("http://localhost:{port}");
    println!("Starting server at {url}");
    open_browser(&url);
    serve(state, port).await
}

fn build_ticker_filter(args: &Args, segments: &Segments) -> Result<Option<HashSet<String>>> {
    if args.all {
        return Ok(None);
    }

    let mut ticker_set = HashSet::new();
    if let Some(path) = args.ticker_csv.as_ref() {
        ticker_set.extend(load_tickers_from_csv(path)?);
    }
    ticker_set.extend(args.tickers.iter().cloned());

    let segment_set = if args.segment.is_empty() {
        None
    } else {
        if segments.is_empty() {
            anyhow::bail!("Error: segments.json not found. Run --fetch-segments first.");
        }
        Some(
            segments
                .iter()
                .filter_map(|(code, segment)| {
                    if args.segment.iter().any(|target| segment.contains(target)) {
                        Some(code.clone())
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>(),
        )
    };

    let result = match (ticker_set.is_empty(), segment_set) {
        (false, Some(segment_set)) => {
            Some(ticker_set.intersection(&segment_set).cloned().collect())
        }
        (false, None) => Some(ticker_set),
        (true, Some(segment_set)) => Some(segment_set),
        (true, None) => None,
    };
    Ok(result)
}
