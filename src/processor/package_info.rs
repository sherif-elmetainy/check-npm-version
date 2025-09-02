use std::collections::BTreeMap;
use reqwest::Client;
use crate::processor::resolve_versions::resolve_versions;
use crate::types::{DependencyType, PackageInfo, PackageJson, PackageJsonReport};
use crate::util;

pub async fn get_package_info(
    client: &Client,
    package_name: &str,
) -> Result<PackageInfo, Box<dyn std::error::Error>> {
    let request = client
        .get(format!("https://registry.npmjs.org/{}", package_name))
        .build()?;

    let response = client.execute(request).await?;
    let text = response.text().await?;
    let package_info: PackageInfo = serde_json::from_str(&text)?;
    Ok(package_info)
}

pub async fn process_package_json(pkg_json_text: String) -> Result<PackageJsonReport, Box<dyn std::error::Error>> {
    // Read and parse package.json
    let mut pkg: PackageJson = serde_json::from_str(&pkg_json_text)?;
    let client = util::build_http_client()?;
    let mut result = PackageJsonReport {
        dependencies: BTreeMap::new(),
    };

    result.dependencies.insert(DependencyType::Normal, vec![]);
    // loop through dependencies
    for (name, version) in std::mem::take(&mut pkg.dependencies) {
        let row = resolve_versions(&client, name, version).await?;
        result.dependencies.get_mut(&DependencyType::Normal).unwrap().push(row);
    }

    result.dependencies.insert(DependencyType::Dev, vec![]);
    for (name, version) in std::mem::take(&mut pkg.dev_dependencies) {
        let row = resolve_versions(&client, name, version).await?;
        result.dependencies.get_mut(&DependencyType::Dev).unwrap().push(row);
    }

    result.dependencies.insert(DependencyType::Peer, vec![]);
    for (name, version) in std::mem::take(&mut pkg.peer_dependencies) {
        let row = resolve_versions(&client, name, version).await?;
        result.dependencies.get_mut(&DependencyType::Peer).unwrap().push(row);
    }

    result.dependencies.insert(DependencyType::Optional, vec![]);
    for (name, version) in std::mem::take(&mut pkg.optional_dependencies) {
        let row = resolve_versions(&client, name, version).await?;
        result.dependencies.get_mut(&DependencyType::Optional).unwrap().push(row);
    }


    Ok(result)
}