use std::path::Path;
use crate::{log_info, log_trace};
use crate::types::PackageManagerType;

pub fn detect_package_manager() -> PackageManagerType {
    if Path::new("yarn.lock").exists() {
        log_trace!("Detected yarn.lock");
        PackageManagerType::Yarn
    }
    else if Path::new("pnpm-lock.yaml").exists() {
        log_trace!("Detected pnpm-lock.yaml");       
        PackageManagerType::Pnpm
    }
    else if Path::new("package-lock.json").exists() {
        log_trace!("Detected package-lock.json");
        PackageManagerType::Npm
    } else {
        log_info!("No package manager detected, assuming npm");
        PackageManagerType::Npm
    }
}
