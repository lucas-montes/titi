use crate::{Configuration, ExecutorSnapshot, LauncherCommand};
use tokio::sync::mpsc;

/// Fuzzer service - communicates with the launcher via channel
pub struct FuzzerService {
    /// Channel to send commands to the launcher
    launcher_tx: mpsc::Sender<LauncherCommand>,
}

impl FuzzerService {
    /// Create a new service with a channel to the launcher
    pub fn new(launcher_tx: mpsc::Sender<LauncherCommand>) -> Self {
        Self { launcher_tx }
    }

    /// Create a new job and return its ID
    pub async fn create_job(&self, config: Configuration) -> Result<u64, String> {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        self.launcher_tx
            .send(LauncherCommand::Create {
                config,
                response: response_tx,
            })
            .await
            .map_err(|_| "Failed to send command to launcher")?;

        response_rx
            .recv()
            .await
            .ok_or_else(|| "No response from launcher".to_string())?
    }

    /// Restart an existing job
    pub async fn restart_job(&self, job_id: u64) -> Result<(), String> {
        // Stop the job first, then create a new one with the same config
        // TODO: Need to store configs to implement restart
        self.stop_job(job_id).await
    }

    /// Stop a running job
    pub async fn stop_job(&self, job_id: u64) -> Result<(), String> {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        self.launcher_tx
            .send(LauncherCommand::Stop {
                job_id,
                response: response_tx,
            })
            .await
            .map_err(|_| "Failed to send command to launcher")?;

        response_rx
            .recv()
            .await
            .ok_or_else(|| "No response from launcher".to_string())?
    }

    /// Get metrics for a job
    pub async fn get_metrics(&self, job_id: u64) -> Result<ExecutorSnapshot, String> {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        self.launcher_tx
            .send(LauncherCommand::GetMetrics {
                job_id,
                response: response_tx,
            })
            .await
            .map_err(|_| "Failed to send command to launcher")?;

        response_rx
            .recv()
            .await
            .ok_or_else(|| "No response from launcher".to_string())?
    }
}
