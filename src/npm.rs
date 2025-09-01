use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct PackageJson {
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    pub dev_dependencies: BTreeMap<String, String>,
    #[serde(rename = "peerDependencies", default)]
    pub peer_dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub time: BTreeMap<String, String>,
}

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
