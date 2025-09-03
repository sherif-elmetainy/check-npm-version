use chrono::DateTime;
use reqwest::Client;
use semver::{Version, VersionReq};
use crate::{log_debug, log_trace, log_warn};
use crate::processor::package_info::get_package_info;
use crate::types::PackageVersionInfo;

pub async fn resolve_versions(
    client: &Client,
    package_name: String,
    declared_version: String,
    npm_config: &crate::processor::npm_config::NpmConfig,
) -> Result<PackageVersionInfo, Box<dyn std::error::Error>> {
    let mut info = get_package_info(client, package_name.as_str(), &npm_config).await?;

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
        log_warn!("Error parsing version req: {} for package {}@{}", e, row.name, row.declared);
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{}: {}", row.declared, e.to_string()),
        )
    })?;
    let declared_version = if row.declared.starts_with('~') || row.declared.starts_with('^') {
        log_debug!("declared version is a range: {}", row.declared);       
        Version::parse(&row.declared[1..])
    } else {
        log_debug!("declared version is a specific version: {}", row.declared);
        Version::parse(row.declared.as_str())
    };
    if declared_version.as_ref().is_ok() {
        if !declared_version.as_ref().unwrap().pre.is_empty() {
            log_debug!("not ignoring pre-release versions for package {}", row.name);       
            ignore_pre_release = false;
        } else {
            log_debug!("ignoring pre-release versions for package {}", row.name);
        }
    }

    for (version, time) in std::mem::take(&mut info.time) {
        log_trace!("Processing version {} at {}", version, time);
        let parsed_version = Version::parse(&version).map_err(|e| {
            log_warn!("Error parsing version: {} for package {}", version, row.name);
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{}: {}", version, e.to_string()),
            )
        })?;
        if ignore_pre_release && !parsed_version.pre.is_empty() {
            log_trace!("Ignoring pre-release version {}", version);
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
        log_trace!("Time: {}", time);

        if row.latest.is_none() || parsed_version.ge(row.latest.as_ref().unwrap()) {
            log_trace!("New latest version: {}", version);
            row.latest = Some(parsed_version.clone());
            row.latest_time = Some(time)
        }

        if declared_req.matches(&parsed_version) {
            log_trace!("Version {} matches declared version {}", version, row.declared);
            if row.latest_satisfying.is_none()
                || parsed_version.ge(row.latest_satisfying.as_ref().unwrap())
            {
                log_trace!("New latest satisfying version: {}", version);
                row.latest_satisfying = Some(parsed_version.clone());
                row.latest_satisfying_time = Some(time)
            } else {
                log_trace!("Latest satisfying version already found: {}", version);
            }
        } else {
            log_trace!("Version {} does not match declared version {}", version, row.declared);
        }

        if declared_version.as_ref().is_ok() {
            log_trace!("Declared version is a specific version: {}", row.declared);
            
            let declared = declared_version.as_ref().unwrap();
            if declared.major == parsed_version.major {
                log_trace!("Major version matches");
                if row.latest_same_major.is_none()
                    || parsed_version.ge(row.latest_same_major.as_ref().unwrap())
                {
                    log_trace!("New latest same major version: {}", version);   
                    row.latest_same_major = Some(parsed_version.clone());
                    row.latest_same_major_time = Some(time)
                }
                if declared.minor == parsed_version.minor {
                    log_trace!("Minor version matches");   
                    if row.latest_same_minor.is_none()
                        || parsed_version.ge(row.latest_same_minor.as_ref().unwrap())
                    {
                        log_trace!("New latest same minor version: {}", version);
                        row.latest_same_minor = Some(parsed_version);
                        row.latest_same_minor_time = Some(time)
                    }
                }
            }
        } else {
            log_trace!("Could not parse declared version {}", row.declared);
        }
    }
    log_trace!("Row: {:?}", row);
    Ok(row)
}
