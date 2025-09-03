use clap::Subcommand;
use crate::types::{PackageManagerType, UpgradeType};

#[derive(Subcommand, Debug)]
pub enum CliSubCommand {
    #[command(
        about = "Prints version information",
        long_about = "Prints version information for this command line tool."
    )]
    Version{},
    #[command(
        about = "Prints a report of the package.json file",
        long_about = "Prints a report of the package.json file."
    )]
    Report {
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
    },
    #[command(
        about = "Upgrades the packages package.json file",
        long_about = "Upgrades the package.json file. If no path is specified, the current directory is used. \
        The path can be a directory or a file. If the path is a directory, the file package.json within the directory is used.\
        If the path is missing and an input is provided via stdin, the input is used."
    )]
    Upgrade {
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
