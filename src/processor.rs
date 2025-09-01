use crate::http::build_http_client;
use crate::npm::{get_package_info, PackageJson};
use crate::report::{print_report_table};
use chrono::{DateTime, Utc};
use reqwest::Client;
use semver::{Version, VersionReq};
use std::path::PathBuf;
use std::{fs};

// Report row describing versions for a package
#[derive(Debug)]
pub struct PackageVersionInfo {
    pub name: String,
    pub declared: String,
    pub latest: Option<Version>,
    pub latest_time: Option<DateTime<Utc>>,
    pub latest_same_major: Option<Version>,
    pub latest_same_major_time: Option<DateTime<Utc>>,
    pub latest_same_minor: Option<Version>,
    pub latest_same_minor_time: Option<DateTime<Utc>>,
    pub latest_satisfying: Option<Version>,
    pub latest_satisfying_time: Option<DateTime<Utc>>,
}
pub async fn process_package(
    client: &Client,
    package_name: String,
    declared_version: String,
) -> Result<PackageVersionInfo, Box<dyn std::error::Error>> {
    let mut info = get_package_info(client, package_name.as_str()).await?;

    info.time.remove("created");
    info.time.remove("modified");

    let mut row = PackageVersionInfo {
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

    let mut ignore_pre_release = true;
    let declared_req = VersionReq::parse(row.declared.as_str()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{}: {}", row.declared, e.to_string()),
        )
    })?;
    let declared_version = if row.declared.starts_with('@') || row.declared.starts_with('^') {
        Version::parse(&row.declared[1..])
    } else {
        Version::parse(row.declared.as_str())
    };
    if declared_version.as_ref().is_ok() {
        if !declared_version.as_ref().unwrap().pre.is_empty() {
            ignore_pre_release = false;
        }
    }

    for (version, time) in std::mem::take(&mut info.time) {
        let parsed_version = Version::parse(&version).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{}: {}", version, e.to_string()),
            )
        })?;
        if ignore_pre_release && !parsed_version.pre.is_empty() {
            continue;
        }
        let time = DateTime::parse_from_rfc3339(time.as_str())
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{}: {}", time, e.to_string()),
                )
            })?
            .to_utc();

        if row.latest.is_none() || parsed_version.ge(row.latest.as_ref().unwrap()) {
            row.latest = Some(parsed_version.clone());
            row.latest_time = Some(time)
        }

        if declared_req.matches(&parsed_version) {
            if row.latest_satisfying.is_none()
                || parsed_version.ge(row.latest_satisfying.as_ref().unwrap())
            {
                row.latest_satisfying = Some(parsed_version.clone());
                row.latest_satisfying_time = Some(time)
            }
        }

        if declared_version.as_ref().is_ok() {
            let declared = declared_version.as_ref().unwrap();
            if declared.major == parsed_version.major {
                if row.latest_same_major.is_none()
                    || parsed_version.ge(row.latest_same_major.as_ref().unwrap())
                {
                    row.latest_same_major = Some(parsed_version.clone());
                    row.latest_same_major_time = Some(time)
                }
                if declared.minor == parsed_version.minor {
                    if row.latest_same_minor.is_none()
                        || parsed_version.ge(row.latest_same_minor.as_ref().unwrap())
                    {
                        row.latest_same_minor = Some(parsed_version);
                        row.latest_same_minor_time = Some(time)
                    }
                }
            }
        };
    }
    Ok(row)
}

pub async fn process_package_json(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse package.json
    let pkg_json_text = fs::read_to_string(&path)?;
    let mut pkg: PackageJson = serde_json::from_str(&pkg_json_text)?;
    let client = build_http_client()?;
    let mut result: Vec<PackageVersionInfo> = Vec::new();

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

    // Render the collected rows as a pretty, colored table
    print_report_table(&result);

    Ok(())
}
