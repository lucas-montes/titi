use crate::config::Configuration;
use crate::executor::ExecutorSnapshot;
use crate::planner::TestPlanner;
use crate::scheduler::Scheduler;
use crate::vuser::Action;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tokio::task::{AbortHandle, JoinSet};

/// Unique identifier for a fuzzer job
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct JobId(u64);

static JOB_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

impl JobId {
    pub fn new() -> Self {
        Self(JOB_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for JobId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Commands that can be sent to the launcher
#[derive(Debug)]
pub enum LauncherCommand {
    /// Create a new job
    Create {
        config: Configuration,
        response: mpsc::Sender<Result<u64, String>>,
    },
    /// Stop a job
    Stop {
        job_id: u64,
        response: mpsc::Sender<Result<(), String>>,
    },
    /// Get metrics for a job
    GetMetrics {
        job_id: u64,
        response: mpsc::Sender<Result<ExecutorSnapshot, String>>,
    },
}

/// Launcher that runs in background and listens for commands
pub struct Launcher {
    command_rx: mpsc::Receiver<LauncherCommand>,
    jobs: HashMap<u64, AbortHandle>,
    metrics: HashMap<u64, ExecutorSnapshot>,
}

impl Launcher {
    /// Create a new launcher with a command channel
    pub fn new(command_rx: mpsc::Receiver<LauncherCommand>) -> Self {
        Self {
            command_rx,
            jobs: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    /// Run the launcher loop - listens for commands
    pub async fn run(mut self) {
        tracing::info!("🚀 Launcher started, listening for commands...");

        while let Some(command) = self.command_rx.recv().await {
            match command {
                LauncherCommand::Create { config, response } => {
                    let result = self.create_job(config).await;
                    let _ = response.send(result).await;
                }

                LauncherCommand::Stop { job_id, response } => {
                    let result = self.stop_job(job_id);
                    let _ = response.send(result).await;
                }

                LauncherCommand::GetMetrics { job_id, response } => {
                    let result = self.get_metrics(job_id);
                    let _ = response.send(result).await;
                }
            }
        }

        tracing::info!("Launcher shutting down");
    }

    /// Create and spawn a new job
    async fn create_job(&mut self, _config: Configuration) -> Result<u64, String> {
        let job_id = JobId::new();

        // TODO: Spawn actual job based on planner type
        // For now, just track that we would spawn it
        tracing::info!("Would create job {}", job_id);

        // Store empty metrics for now
        self.metrics.insert(job_id.as_u64(), todo!("Create empty snapshot"));

        Ok(job_id.as_u64())
    }

    /// Stop a running job
    fn stop_job(&mut self, job_id: u64) -> Result<(), String> {
        if let Some(handle) = self.jobs.remove(&job_id) {
            handle.abort();
            tracing::info!("Stopped job {}", job_id);
            Ok(())
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    /// Get metrics for a job
    fn get_metrics(&self, job_id: u64) -> Result<ExecutorSnapshot, String> {
        self.metrics
            .get(&job_id)
            .cloned()
            .ok_or_else(|| format!("No metrics for job {}", job_id))
    }
}
