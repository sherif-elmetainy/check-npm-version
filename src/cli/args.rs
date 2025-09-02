use clap::Parser;
use crate::cli::sub_commands::CliSubCommand;
use crate::util::LogLevel;

#[derive(Parser, Debug)]
pub struct CliArgs {
    #[arg(
        short,
        long,
        default_value = ".",
        help = "The path to the package.json file. \
        If not specified otherwise the package.json file of current directory is used.\
        If path is a directory, the file package.json within the directory is used.\
        "
    )]
    pub path: String,
    #[arg(
        short,
        long,
        help = "Set the logging verbosity level",
        long_help = "Set the logging verbosity level:\n\
        - off: No logging output\n\
        - error: Only errors that prevent normal operation\n\
        - warn: Warnings about potential issues (default)\n\
        - info: General information about program execution\n\
        - debug: Detailed information useful for debugging\n\
        - trace: Very detailed tracing information"
    )]
    pub log_level: Option<LogLevel>,
    #[command(subcommand)]
    pub command: CliSubCommand,
}
