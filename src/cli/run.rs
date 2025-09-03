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
    let log_level = args.log_level.unwrap_or(crate::util::LogLevel::default());
    set_log_level(log_level);
    log_info!("Log level set to {}", get_log_level());

    
    match args.command {
        CliSubCommand::Report{path, skip_latest} => {
            let report = get_report(path.as_str()).await?;
            print_report(&report, skip_latest);
        }
        CliSubCommand::Upgrade {
            path,
            upgrade_type,
            package_manager,
        } => {
            let report = get_report(path.as_str()).await?;
            upgrade_packages(path, report, package_manager, upgrade_type)?;
        },
        CliSubCommand::Version{} => {
            println!("{} v{} ({})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
        }
    }

    Ok(())
}