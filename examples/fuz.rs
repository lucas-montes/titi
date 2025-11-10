use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;
use std::env;
use std::path::PathBuf;

use fuzzer::{configuration::Configuration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    // let args: Vec<String> = env::args().collect();

    // if args.len() != 2 {
    //     eprintln!("Usage: {} <config-file.json>", args[0]);
    //     eprintln!("Example: {} examples/load_test_constant.json", args[0]);
    //     std::process::exit(1);
    // }

    // let config_path = PathBuf::from(&args[1]);

    let config_path = PathBuf::from("examples/test-scenarios/load_test_constant.json");

    // Read and parse the configuration file
    tracing::debug!("Reading configuration from: {}", config_path.display());
    let config_content = tokio::fs::read_to_string(&config_path).await?;
    let config: Configuration = serde_json::from_str(&config_content)?;

    tracing::debug!(?config,"Configuration loaded successfully");

    Ok(())
}
