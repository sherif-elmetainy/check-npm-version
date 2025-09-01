use std::collections::{BTreeMap};
use std::{env, fs};
use std::path::{Path, PathBuf};
use std::time::{Duration};
use chrono::{DateTime, Utc};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use semver::{Version, VersionReq};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    dev_dependencies: BTreeMap<String, String>,
    #[serde(rename = "peerDependencies", default)]
    peer_dependencies: BTreeMap<String, String>,
}
//static TIME_FORMAT: LazyLock<Vec<time::format_description::FormatItem>> = LazyLock::new(|| time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second].[millisecond]Z").unwrap());
#[derive(Debug, Deserialize)]
struct PackageInfo {
    time: BTreeMap<String, String>,
}

#[derive(Debug)]
struct PackageReportRow {
    name: String,
    declared: String,
    latest: Option<Version>,
    latest_time: Option<DateTime<Utc>>,
    latest_same_major: Option<Version>,
    latest_same_major_time: Option<DateTime<Utc>>,
    latest_same_minor: Option<Version>,
    latest_same_minor_time: Option<DateTime<Utc>>,
    latest_satisfying: Option<Version>,
    latest_satisfying_time: Option<DateTime<Utc>>,
}

fn resolve_package_path() -> Result<PathBuf, std::io::Error> {
    let mut arg = env::args().nth(1).unwrap_or_else(String::new);

    if arg.trim().is_empty() {
        arg = ".".to_string()
    }
    let path = Path::new(&arg);
    let metadata = path.metadata().map_err(
        |e| std::io::Error::new(e.kind(), format!("path {} is not valid: {}", arg, e.to_string()))
    )?;

    let file_path = if metadata.is_dir() {
        let path = path.join("package.json");
        let metadata = path.metadata().map_err(
            |e| std::io::Error::new(e.kind(), format!("path {} is not valid: {}", path.to_str().unwrap(), e.to_string()))
        )?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("path {} is not valid: {}", path.to_str().unwrap(), "package.json is not a file")))
        }
        path
    } else {
        path.to_path_buf()
    };

    Ok(file_path)
}

fn build_http_client() -> Result<Client, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str("check-npm-version/1.0 (+https://github.com/sherif-elmetainy/check-npm-version)")?,
    );
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(20))
        .build()?;
    Ok(client)
}

async fn get_package_info(client: &Client, package_name: &str) -> Result<PackageInfo, Box<dyn std::error::Error>> {
    let request= client.get(format!("https://registry.npmjs.org/{}", package_name)).build()?;

    let response = client.execute(request).await?;
    let text = response.text().await?;
    let package_info: PackageInfo = serde_json::from_str(&text)?;
    Ok(package_info)
}

async fn process_package(client: &Client, package_name: String, declared_version: String) -> Result<PackageReportRow, Box<dyn std::error::Error>> {
    let mut info = get_package_info(client, package_name.as_str()).await?;


    info.time.remove("created");
    info.time.remove("modified");

    let mut row = PackageReportRow {
        name: package_name,
        declared: declared_version,
        latest: None,
        latest_time: None,
        latest_same_major: None,
        latest_same_major_time: None,
        latest_same_minor: None,
        latest_same_minor_time: None,
        latest_satisfying: None,
        latest_satisfying_time: None,
    };

    let declared_req = VersionReq::parse(row.declared.as_str()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}: {}", row.declared, e.to_string())))?;
    let declared_version = Version::parse(row.declared.as_str());

    
    for (version, time) in std::mem::take(&mut info.time) {
        let parsed_version = Version::parse(&version).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}: {}", version, e.to_string())))?;
        let time = DateTime::parse_from_rfc3339(time.as_str()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}: {}", time, e.to_string())))?.to_utc();

        if row.latest.is_none() || parsed_version.ge(row.latest.as_ref().unwrap()) {
            row.latest = Some(parsed_version.clone());
            row.latest_time = Some(time)
        }

        if declared_req.matches(&parsed_version) {
            if row.latest_satisfying.is_none() || parsed_version.ge(row.latest_satisfying.as_ref().unwrap()) {
                row.latest_satisfying = Some(parsed_version.clone());
                row.latest_satisfying_time = Some(time)
            }
        }

        if declared_version.as_ref().is_ok() {
            let declared = declared_version.as_ref().unwrap();
            if declared.major == parsed_version.major {
                if row.latest_same_major.is_none() || parsed_version.ge(row.latest_same_major.as_ref().unwrap()) {
                    row.latest_same_major = Some(parsed_version.clone());
                    row.latest_same_major_time = Some(time)
                }
                if declared.minor == parsed_version.minor {
                    if row.latest_same_minor.is_none() || parsed_version.ge(row.latest_same_minor.as_ref().unwrap()) {
                        row.latest_same_minor = Some(parsed_version);
                        row.latest_same_minor_time = Some(time)
                    }
                }
            }
        };
    }
    Ok(row)


}

async fn process_package_json(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 2) Read and parse package.json
    let pkg_json_text = fs::read_to_string(&path)?;
    let mut pkg: PackageJson = serde_json::from_str(&pkg_json_text)?;
    let client = build_http_client()?;
    let mut result : Vec<PackageReportRow> = Vec::new();

    // loop through dependencies
    for (name, version) in std::mem::take(&mut pkg.dependencies) {
        let row = process_package(&client, name, version).await?;
        result.push(row);
    }

    for (name, version) in std::mem::take(&mut pkg.dev_dependencies) {
        let row = process_package(&client, name, version).await?;
        result.push(row);
    }

    for (name, version) in std::mem::take(&mut pkg.peer_dependencies) {
        let row = process_package(&client, name, version).await?;
        result.push(row);
    }

    Ok(())
}


#[tokio::main]
async fn main() {

    let path = resolve_package_path();
    match path {
        Ok(file) => process_package_json(file.clone()).await.unwrap_or_else(
            |e| println!("An error occurred while processing file {}: {}", file.to_str().unwrap(), e)
        ),
        Err(e) => println!("{}", e)
    }
}
