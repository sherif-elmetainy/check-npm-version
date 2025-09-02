use crate::processor::process_package_json;
use crate::report::print_report;
use crate::upgrade::{PackageManagerType, UpgradeType, upgrade_packages};
use clap::{Parser, Subcommand};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

#[derive(Subcommand, Debug)]
enum SubCommand {
    #[command(
        about = "Prints a report of the package.json file",
        long_about = "Prints a report of the package.json file."
    )]
    Report {
    },
    #[command(
        about = "Upgrades the packages package.json file",
        long_about = "Upgrades the package.json file. If no path is specified, the current directory is used. \
        The path can be a directory or a file. If the path is a directory, the file package.json within the directory is used.\
        If the path is missing and an input is provided via stdin, the input is used."
    )]
    Upgrade {
        #[arg(
            long = "type",
            short = 't',
            value_enum,
            long_help = "The type of upgrade to perform (does not respect semver ranges). \
        Latest: upgrade to the latest version even if it is a major or minor upgrade. \
        Major: upgrade to the latest major version that matches the semver range. \
        Minor: upgrade to the latest minor version that matches the semver range.  \
        "
        )]
        upgrade_type: Option<UpgradeType>,
        #[arg(
            short = 'm',
            value_enum,
            long_help = "The package manager to use. \
        npm: use npm to upgrade packages. \
        yarn: use yarn to upgrade packages. \
        pnpm: use pnpm to upgrade packages.
        "
        )]
        package_manager: Option<PackageManagerType>,
    },
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(
        short,
        long,
        default_value = ".",
        help = "The path to the package.json file. \
        If not specified otherwise the package.json file of current directory is used.\
        If path is a directory, the file package.json within the directory is used.\
        "
    )]
    path: String,
    #[command(subcommand)]
    command: SubCommand,
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    let path = args.path;
    let pkg_json_text = get_package_json(path.as_str())?;
    let report = process_package_json(pkg_json_text).await?;
    match args.command {
        SubCommand::Report{} => {
            print_report(&report);
        }
        SubCommand::Upgrade {
            upgrade_type,
            package_manager,
        } => {
            upgrade_packages(path, report, package_manager, upgrade_type)?;
        }
    }

    Ok(())
}

fn get_package_json(path: &str) -> Result<String, io::Error> {
    if path.trim().is_empty() {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            // read stdin
            let mut buf = String::new();
            stdin.lock().read_to_string(&mut buf).map_err(|e| {
                io::Error::new(e.kind(), format!("failed to read stdin: {}", e.to_string()))
            })?;
            return Ok(buf);
        }
    }

    let path = resolve_package_path(path)?;
    let file_path = path.to_str().unwrap();
    let mut file = File::open(file_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to open file {}: {}", file_path, e.to_string()),
        )
    })?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to read file {}: {}", file_path, e.to_string()),
        )
    })?;

    Ok(buf)
}

fn resolve_package_path(arg: &str) -> Result<PathBuf, io::Error> {
    let path = Path::new(&arg);
    let metadata = path.metadata().map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("path {} is not valid: {}", arg, e.to_string()),
        )
    })?;

    let file_path = if metadata.is_dir() {
        let path = path.join("package.json");
        let metadata = path.metadata().map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "path {} is not valid: {}",
                    path.to_str().unwrap(),
                    e.to_string()
                ),
            )
        })?;
        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
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
