use std::time::Duration;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use crate::log_trace;

pub fn build_http_client() -> Result<Client, Box<dyn std::error::Error>> {
    log_trace!("Building HTTP client");
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(
            "check-npm-version/1.0 (+https://github.com/sherif-elmetainy/check-npm-version)",
        )?,
    );
    
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()?;
    log_trace!("HTTP client built");
    Ok(client)
}