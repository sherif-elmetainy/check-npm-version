mod cli;
mod http;
mod npm;
mod report;
mod processor;
mod upgrade;

#[tokio::main]
async fn main() {
    let report = cli::run().await;
    if let Err(e) = report {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
