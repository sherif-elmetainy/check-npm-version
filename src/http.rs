use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use std::time::Duration;

pub fn build_http_client() -> Result<Client, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(
            "check-npm-version/1.0 (+https://github.com/sherif-elmetainy/check-npm-version)",
        )?,
    );
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(20))
        .build()?;
    Ok(client)
}
