mod cli;
mod report;
mod processor;
mod upgrade;
mod util;
mod types;

#[tokio::main]
async fn main() {
    let report = cli::run().await;
    if let Err(e) = report {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
