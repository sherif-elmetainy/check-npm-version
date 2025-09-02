use std::collections::BTreeMap;
use crate::types::dependency_type::DependencyType;
use crate::types::package_version_info::PackageVersionInfo;

pub struct PackageJsonReport {
    pub dependencies: BTreeMap<DependencyType, Vec<PackageVersionInfo>>,
}