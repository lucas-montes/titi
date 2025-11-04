use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;
use std::env;
use std::path::PathBuf;

use fuzzer::{Launcher, LauncherCommand, Configuration};

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

    // Setup launcher
    let (launcher_tx, launcher_rx) = mpsc::channel(100);

    let launcher = Launcher::new(launcher_rx);
    let launcher_handle = tokio::spawn(async move {
        tracing::info!("Starting fuzzer launcher...");
        launcher.run().await;
    });

    // Parse command line arguments
    // let args: Vec<String> = env::args().collect();

    // if args.len() != 2 {
    //     eprintln!("Usage: {} <config-file.json>", args[0]);
    //     eprintln!("Example: {} examples/load_test_constant.json", args[0]);
    //     std::process::exit(1);
    // }

    // let config_path = PathBuf::from(&args[1]);

    let config_path = PathBuf::from("examples/load_test_constant.json");

    // Read and parse the configuration file
    tracing::info!("Reading configuration from: {}", config_path.display());
    let config_content = tokio::fs::read_to_string(&config_path).await?;
    let config: Configuration = serde_json::from_str(&config_content)?;

    tracing::info!("Configuration loaded successfully");

    // Send create job command to launcher
    let (response_tx, mut response_rx) = mpsc::channel(1);
    launcher_tx
        .send(LauncherCommand::Create {
            config,
            response: response_tx,
        })
        .await?;

    // Wait for job creation response
    match response_rx.recv().await {
        Some(Ok(job_id)) => {
            tracing::info!("✅ Job created successfully with ID: {}", job_id);

            // TODO: Keep running and monitor the job
            // For now, just wait a bit and then shutdown
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            tracing::info!("Shutting down...");
        }
        Some(Err(e)) => {
            tracing::error!("❌ Failed to create job: {}", e);
            return Err(e.into());
        }
        None => {
            tracing::error!("❌ Launcher closed unexpectedly");
            return Err("Launcher closed".into());
        }
    }

    // Close the launcher command channel
    drop(launcher_tx);

    // Wait for launcher to finish
    launcher_handle.await?;

    Ok(())
}
