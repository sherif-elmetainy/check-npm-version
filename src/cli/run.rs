use std::error::Error;
use clap::Parser;
use crate::cli::args::CliArgs;
use crate::cli::sub_commands::CliSubCommand;
use crate::{log_info};
use crate::processor::get_report;
use crate::report::print_report;
use crate::upgrade::upgrade_packages;
use crate::util::{get_log_level, set_log_level};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    let path = args.path;
    let log_level = args.log_level.unwrap_or(crate::util::LogLevel::default());
    set_log_level(log_level);
    log_info!("Path set to {}", path);
    log_info!("Log level set to {}", get_log_level());

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