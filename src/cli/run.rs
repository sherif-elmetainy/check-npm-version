use std::error::Error;
use clap::Parser;
use crate::cli::args::CliArgs;
use crate::cli::sub_commands::CliSubCommand;
use crate::processor::get_report;
use crate::report::print_report;
use crate::upgrade::upgrade_packages;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    let path = args.path;
    let report = get_report(path.as_str()).await?;
    match args.command {
        CliSubCommand::Report{} => {
            print_report(&report);
        }
        CliSubCommand::Upgrade {
            upgrade_type,
            package_manager,
        } => {
            upgrade_packages(path, report, package_manager, upgrade_type)?;
        }
    }

    Ok(())
}