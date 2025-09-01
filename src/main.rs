mod cli;
mod http;
mod npm;
mod report;
mod processor;

#[tokio::main]
async fn main() {
    let path = crate::cli::resolve_package_path();
    match path {
        Ok(file) => crate::processor::process_package_json(file.clone())
            .await
            .unwrap_or_else(|e| {
                println!(
                    "An error occurred while processing file {}: {}",
                    file.to_str().unwrap(),
                    e
                )
            }),
        Err(e) => println!("{}", e),
    }
}
