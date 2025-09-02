use crate::types::{DependencyType, PackageManagerType};

pub fn get_install_args(package_manager: &PackageManagerType, dependency_type: DependencyType) -> Vec<String> {
    match (package_manager, dependency_type) {
        ( PackageManagerType::Npm, DependencyType::Dev) => vec!["install".to_string(), "--save-dev".to_string()],
        ( PackageManagerType::Npm, DependencyType::Normal) => vec!["install".to_string(), "--save-prod".to_string()],
        ( PackageManagerType::Npm, DependencyType::Peer) => vec!["install".to_string(), "--save-peer".to_string()],
        ( PackageManagerType::Npm, DependencyType::Optional) => vec!["install".to_string(), "--save-optional".to_string()],

        ( PackageManagerType::Yarn, DependencyType::Dev) => vec!["add".to_string(), "--dev".to_string()],
        ( PackageManagerType::Yarn, DependencyType::Normal) => vec!["add".to_string()],
        ( PackageManagerType::Yarn, DependencyType::Peer) => vec!["add".to_string(), "--peer".to_string()],
        ( PackageManagerType::Yarn, DependencyType::Optional) => vec!["add".to_string(), "--optional".to_string()],

        ( PackageManagerType::Pnpm, DependencyType::Dev) => vec!["add".to_string(), "--save-dev".to_string()],
        ( PackageManagerType::Pnpm, DependencyType::Normal) => vec!["add".to_string(), "--save-prod".to_string()],
        ( PackageManagerType::Pnpm, DependencyType::Peer) => vec!["add".to_string(), "--save-peed".to_string()],
        ( PackageManagerType::Pnpm, DependencyType::Optional) => vec!["add".to_string(), "--save-optional".to_string()],
    }
}