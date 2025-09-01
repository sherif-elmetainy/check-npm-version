use std::env;
use std::path::{Path, PathBuf};

pub fn resolve_package_path() -> Result<PathBuf, std::io::Error> {
    let mut arg = env::args().nth(1).unwrap_or_else(String::new);

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
