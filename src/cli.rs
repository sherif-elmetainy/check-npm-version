use std::{env, io};
use std::error::Error;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use crate::processor::process_package_json;
use crate::report::print_report;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let pkg_json_text = get_package_json()?;
    let report = process_package_json(pkg_json_text).await?;
    
    print_report(&report);
    
    Ok(())
}

fn get_package_json() -> Result<String, std::io::Error> {
    let arg = env::args().nth(1).unwrap_or_else(String::new);
    if arg.trim().is_empty() {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            // read stdin
            let mut buf = String::new();
            stdin.lock().read_to_string(&mut buf).map_err(|e| { std::io::Error::new(e.kind(), format!("failed to read stdin: {}", e.to_string())) })?;
            return Ok(buf)
        }

    }

    let path = resolve_package_path(arg)?;
    let file_path = path.to_str().unwrap();
    let mut file = std::fs::File::open(file_path).map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("failed to open file {}: {}", file_path, e.to_string()),
        )
    })?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("failed to read file {}: {}", file_path, e.to_string()),
        )
    })?;

    Ok(buf)

}

fn resolve_package_path(mut arg: String) -> Result<PathBuf, std::io::Error> {

    if arg.trim().is_empty() {
        arg = ".".to_string()
    }
    let path = Path::new(&arg);
    let metadata = path.metadata().map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("path {} is not valid: {}", arg, e.to_string()),
        )
    })?;

    let file_path = if metadata.is_dir() {
        let path = path.join("package.json");
        let metadata = path.metadata().map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!(
                    "path {} is not valid: {}",
                    path.to_str().unwrap(),
                    e.to_string()
                ),
            )
        })?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
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
