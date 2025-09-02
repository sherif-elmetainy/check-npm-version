use crate::types::{PackageVersionInfo, UpgradeType};

pub fn get_update_list(packages: Vec<PackageVersionInfo>, upgrade_type: &UpgradeType) -> Vec<String> {
    let mut update_list = vec![];

    for package in packages {
        let version = match upgrade_type {
            UpgradeType::Major => {
                package.latest_same_major
            },
            UpgradeType::Minor => {
                package.latest_same_minor
            },
            UpgradeType::Latest => {
                package.latest
            },
        };
        if let Some(version) = version {
            let (spec, current) = if package.declared.starts_with('@') {
                ("@", &package.declared[1..])
            } else if package.declared.starts_with('^') {
                ("^", &package.declared[1..])
            } else {
                ("", package.declared.as_str())
            };
            if current.eq(version.to_string().as_str()) {
                continue;
            }
            update_list.push(format!("{}@{}{}", package.name, spec, version));
        }
    }

    update_list
}
