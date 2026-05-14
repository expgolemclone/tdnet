use std::time::Duration;

use anyhow::{Context, Result};
use bytes::Bytes;
use reqwest::{Client, Response, StatusCode};
use tokio::time::sleep;

use crate::config::{RETRY_BACKOFF_MS, RETRY_TOTAL};

#[derive(Clone)]
pub struct TdnetClient {
    inner: Client,
}

impl TdnetClient {
    pub fn new() -> Result<Self> {
        let inner = Client::builder()
            .user_agent("tdnet-viewer/0.1")
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self { inner })
    }

    pub async fn get_text(&self, url: &str, timeout_secs: u64) -> Result<String> {
        let bytes = self.get_bytes(url, timeout_secs).await?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub async fn get_bytes_checked(&self, url: &str, timeout_secs: u64) -> Result<Bytes> {
        let resp = self.get_response(url, timeout_secs).await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("GET {url} failed with HTTP {status}");
        }
        resp.bytes()
            .await
            .with_context(|| format!("failed to read response body from {url}"))
    }

    pub async fn get_bytes(&self, url: &str, timeout_secs: u64) -> Result<Bytes> {
        self.get_response(url, timeout_secs)
            .await?
            .bytes()
            .await
            .with_context(|| format!("failed to read response body from {url}"))
    }

    async fn get_response(&self, url: &str, timeout_secs: u64) -> Result<Response> {
        let mut last_error = None;
        for attempt in 0..RETRY_TOTAL {
            let result = self
                .inner
                .get(url)
                .timeout(Duration::from_secs(timeout_secs))
                .send()
                .await;

            match result {
                Ok(resp) if should_retry(resp.status()) && attempt + 1 < RETRY_TOTAL => {
                    sleep(backoff(attempt)).await;
                    continue;
                }
                Ok(resp) => return Ok(resp),
                Err(err) if attempt + 1 < RETRY_TOTAL => {
                    last_error = Some(err);
                    sleep(backoff(attempt)).await;
                }
                Err(err) => {
                    return Err(err).with_context(|| format!("GET {url} failed"));
                }
            }
        }

        Err(last_error.expect("retry loop should keep last error"))
            .with_context(|| format!("GET {url} failed"))
    }
}

fn should_retry(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::BAD_GATEWAY | StatusCode::SERVICE_UNAVAILABLE | StatusCode::GATEWAY_TIMEOUT
    )
}

fn backoff(attempt: usize) -> Duration {
    Duration::from_millis(RETRY_BACKOFF_MS * (attempt as u64 + 1))
}
