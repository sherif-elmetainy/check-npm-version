use crate::{log_debug, log_info, log_trace, log_warn};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::ClientBuilder;
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use std::path::PathBuf;
use std::time::Duration;
fn get_cache_dir() -> PathBuf {
    let dir = dirs::cache_dir();
    if let Some(d) = dir {
        log_info!("Cache dir: {}", d.display());
        return d;
    }
    let dir = std::env::temp_dir();
    log_warn!("failed to get cache dir, using temp dir instead: {}", dir.display());
    dir
}

fn get_cache_dir_full() -> PathBuf {
    let mut dir = get_cache_dir().join("code-art").join("check-npm-version");
    if cfg!(windows) {
        dir = dir.join("cache");
    }

    if !dir.exists() {
        log_info!("Creating cache dir: {}", dir.display());
        std::fs::create_dir_all(&dir).unwrap();
        log_trace!("Cache dir created");
    }
    log_debug!("Cache dir: {}", dir.display());
    dir
}


pub fn build_http_client() -> Result<ClientWithMiddleware, Box<dyn std::error::Error>> {
    log_trace!("Building HTTP client");
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(
            "check-npm-version/1.0 (+https://github.com/sherif-elmetainy/check-npm-version)",
        )?,
    );

    let cache_less = ClientBuilder::new()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()?
        ;

    
    let client = MiddlewareClientBuilder::new(cache_less)
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager { path: get_cache_dir_full(), remove_opts: Default::default() },
            options: HttpCacheOptions::default(),
        })).build();
    log_trace!("HTTP client built");
    Ok(client)
}