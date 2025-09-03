use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::{log_debug, log_trace, log_warn};
use crate::processor::npm_config::NpmConfig;
use crate::processor::package_info::process_package_json;
use crate::types::PackageJsonReport;

pub async fn get_report(path: &str) -> Result<PackageJsonReport, Box<dyn std::error::Error>> {
    log_debug!("Processing package.json at {}", path);
    let pkg_json_text = get_package_json(path)?;
    log_trace!("Package.json text: {}", pkg_json_text);

    // load .npmrc
    let mut npm_config = NpmConfig::new();
    npm_config.load_for_project(path)?;

    let report = process_package_json(pkg_json_text, &npm_config).await?;
    log_trace!("Report: {:?}", report);
    Ok(report)
}


fn get_package_json(path: &str) -> Result<String, io::Error> {
    
    let path = resolve_package_path(path)?;
    let file_path = path.to_str().unwrap();
    let mut file = File::open(file_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to open file {}: {}", file_path, e.to_string()),
        )
    })?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to read file {}: {}", file_path, e.to_string()),
        )
    })?;

    Ok(buf)
}

fn resolve_package_path(arg: &str) -> Result<PathBuf, io::Error> {
    let path = Path::new(&arg);
    let metadata = path.metadata().map_err(|e| {
        log_warn!("Error: {}", e.to_string());
        io::Error::new(
            e.kind(),
            format!("path {} is not valid: {}", arg, e.to_string()),
        )
    })?;

    let file_path = if metadata.is_dir() {
        let path = path.join("package.json");
        let metadata = path.metadata().map_err(|e| {
            log_warn!("Error: {}", e.to_string());
            io::Error::new(
                e.kind(),
                format!(
                    "path {} is not valid: {}",
                    path.to_str().unwrap(),
                    e.to_string()
                ),
            )
        })?;
        if !metadata.is_file() {
            log_warn!("package.json is not a file");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "path {} is not valid: {}",
                    path.to_str().unwrap(),
                    "package.json is not a file"
                ),
            ));
        }
        path
    } else {
        path.to_path_buf()
    };

    Ok(file_path)
}