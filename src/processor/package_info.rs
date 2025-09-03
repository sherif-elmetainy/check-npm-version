use std::collections::BTreeMap;
use reqwest::Client;
use crate::processor::resolve_versions::resolve_versions;
use crate::types::{DependencyType, PackageInfo, PackageJson, PackageJsonReport};
use crate::{log_debug, log_error, log_info, log_trace, util};
use crate::processor::npm_config::NpmConfig;

pub async fn get_package_info(
    client: &Client,
    package_name: &str,
    npm_config: &NpmConfig,
) -> Result<PackageInfo, Box<dyn std::error::Error>> {
    log_info!("Getting package info for {}", package_name);
    let registry = npm_config.get_package_registry(package_name);
    log_debug!("Registry: {} for package {}", registry, package_name);
    let mut request_builder = client
        .get(format!("{}{}", registry, package_name))
        ;
    
    let auth_token = npm_config.get_auth_token(registry);
    if let Some(auth_token) = auth_token {
        log_debug!("Using auth token for registry {}", registry);
        request_builder = request_builder.bearer_auth(auth_token);
    }
    let request = request_builder.build()?;
    log_debug!("Request: {}", request.url());

    let response = client.execute(request).await?;
    log_debug!("Response: {}", response.status());
    let status = response.status();
    if !status.is_success() {
        log_error!("Response code {}: {}", status, response.text().await?);
        return Err(format!("Failed to get package info for {}: {}", package_name, status).into());
    }
    let text = response.text().await?;
    log_trace!("Text: {}", text);
    let package_info: PackageInfo = serde_json::from_str(&text)?;
    log_trace!("Package info: {:?}", package_info);
    Ok(package_info)
}

pub async fn process_package_json(pkg_json_text: String, npm_config: &NpmConfig) -> Result<PackageJsonReport, Box<dyn std::error::Error>> {
    // Read and parse package.json
    log_info!("Processing package.json");
    let mut pkg: PackageJson = serde_json::from_str(&pkg_json_text)?;
    log_trace!("Package json: {:?}", pkg);
    let client = util::build_http_client()?;
    let mut result = PackageJsonReport {
        dependencies: BTreeMap::new(),
    };

    log_info!("Resolving versions for dependencies");

    result.dependencies.insert(DependencyType::Normal, vec![]);
    // loop through dependencies
    for (name, version) in std::mem::take(&mut pkg.dependencies) {
        log_trace!("Processing dependency: {} {}", name, version);
        let row = resolve_versions(&client, name, version, &npm_config).await?;
        result.dependencies.get_mut(&DependencyType::Normal).unwrap().push(row);
    }

    result.dependencies.insert(DependencyType::Dev, vec![]);
    for (name, version) in std::mem::take(&mut pkg.dev_dependencies) {
        log_trace!("Processing dev dependency: {} {}", name, version);
        let row = resolve_versions(&client, name, version, &npm_config).await?;
        result.dependencies.get_mut(&DependencyType::Dev).unwrap().push(row);
    }

    result.dependencies.insert(DependencyType::Peer, vec![]);
    for (name, version) in std::mem::take(&mut pkg.peer_dependencies) {
        log_trace!("Processing peer dependency: {} {}", name, version);
        let row = resolve_versions(&client, name, version, &npm_config).await?;
        result.dependencies.get_mut(&DependencyType::Peer).unwrap().push(row);
    }

    result.dependencies.insert(DependencyType::Optional, vec![]);
    for (name, version) in std::mem::take(&mut pkg.optional_dependencies) {
        log_trace!("Processing optional dependency: {} {}", name, version);
        let row = resolve_versions(&client, name, version, &npm_config).await?;
        result.dependencies.get_mut(&DependencyType::Optional).unwrap().push(row);
    }

    Ok(result)
}