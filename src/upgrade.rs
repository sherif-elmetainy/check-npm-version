use std::fmt::Display;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::processor::{PackageJsonReport, PackageVersionInfo};

struct Defer<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Defer<F> {
    fn new(f: F) -> Self { Defer(Some(f)) }
}
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}


#[derive(clap::ValueEnum, Clone, Debug)]
pub enum UpgradeType {
    Major,
    Minor,
    Latest,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum DependencyType {
    Normal,
    Dev,
    Peer,
    Optional,
}

impl Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Normal => write!(f, "Normal Dependencies"),
            DependencyType::Dev => write!(f, "Development Dependencies"),
            DependencyType::Peer => write!(f, "Peer Dependencies"),
            DependencyType::Optional => write!(f, "Optional Dependencies"),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PackageManagerType {
    Npm,
    Yarn,
    Pnpm,
}

impl Display for PackageManagerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManagerType::Npm => write!(f, "npm"),
            PackageManagerType::Yarn => write!(f, "yarn"),
            PackageManagerType::Pnpm => write!(f, "pnpm"),
        }
    }
}


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

fn get_install_args(package_manager: &PackageManagerType, dependency_type: DependencyType) -> Vec<String> {
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

fn get_update_list(packages: Vec<PackageVersionInfo>, upgrade_type: &UpgradeType) -> Vec<String> {
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

fn detect_package_manager() -> PackageManagerType {
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