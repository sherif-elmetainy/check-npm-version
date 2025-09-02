use std::collections::BTreeMap;
use crate::types::dependency_type::DependencyType;
use crate::types::package_version_info::PackageVersionInfo;

#[derive(Debug)]
pub struct PackageJsonReport {
    pub dependencies: BTreeMap<DependencyType, Vec<PackageVersionInfo>>,
}