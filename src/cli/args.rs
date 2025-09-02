use clap::Parser;
use crate::cli::sub_commands::CliSubCommand;

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
    #[command(subcommand)]
    pub command: CliSubCommand,
}
