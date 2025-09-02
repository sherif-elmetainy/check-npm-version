use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::types::{PackageJsonReport, PackageManagerType, UpgradeType};
use crate::upgrade::install_args::get_install_args;
use crate::upgrade::package_manager::detect_package_manager;
use crate::upgrade::update_list::get_update_list;
use crate::util::Defer;

pub fn upgrade_packages(path: String, package_json_report: PackageJsonReport, package_manager: Option<PackageManagerType>, upgrade_type: Option<UpgradeType>) -> Result<(), Box<dyn std::error::Error>> {
    let upgrade_type = upgrade_type.unwrap_or(UpgradeType::Major);
    let path = resolve_package_json_folder(path.as_str())?;
    let current_dir = std::env::current_dir()?;

    std::env::set_current_dir(path)?;
    let _cleanup = Defer::new(|| {
        let _ = std::env::set_current_dir(current_dir);
    });
    let package_manager = package_manager.unwrap_or(detect_package_manager());

    do_upgrade(package_manager, upgrade_type, package_json_report)?;

    Ok(())
}

fn do_upgrade(package_manager: PackageManagerType, upgrade_type: UpgradeType, package_json_report: PackageJsonReport) -> Result<(), Box<dyn std::error::Error>> {
    for (dependency_type, packages) in package_json_report.dependencies {
        let mut args = get_install_args(&package_manager, dependency_type);
        let mut update_list = get_update_list(packages, &upgrade_type);
        if update_list.is_empty() {
            continue;
        }

        args.append(&mut update_list);

        let status = Command::new(&package_manager.to_string())
            .args(&args)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .stdin(std::process::Stdio::inherit())
            .status()?
            ;
        if !status.success() {
            return Err(format!("{} failed with exit code {}", package_manager, status.code().unwrap()).into());
        }
    };

    Ok(())
}

fn resolve_package_json_folder(arg: &str) -> Result<PathBuf, io::Error> {
    let path = Path::new(&arg);
    let md = path.metadata()?;
    let folder_path = if !md.is_dir() {
        let parent = path.parent();
        if parent.is_none() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "path is not valid"));
        }
        parent.unwrap().to_path_buf()
    } else {
        path.to_path_buf()
    };

    Ok(folder_path)
}
