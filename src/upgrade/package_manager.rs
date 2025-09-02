use std::path::Path;
use crate::types::PackageManagerType;

pub fn detect_package_manager() -> PackageManagerType {
    if Path::new("yarn.lock").exists() {
        PackageManagerType::Yarn
    }
    else if Path::new("pnpm-lock.yaml").exists() {
        PackageManagerType::Pnpm
    }
    else {
        PackageManagerType::Npm
    }
}
