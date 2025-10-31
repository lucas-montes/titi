use serde::{Deserialize, Serialize};

use crate::{Configuration, ExecutorSnapshot};

/// POST request for job operations
#[derive(Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "lowercase")]
pub enum JobOperation {
    /// Create a new fuzzer job
    Create { config: Configuration },

    /// Restart an existing job
    Restart { job_id: u64 },

    /// Stop a running job
    Stop { job_id: u64 },
}

/// Response for job operations (POST)
#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "lowercase")]
pub enum JobOperationResponse {
    /// Job created successfully
    Created { job_id: u64 },

    /// Job restarted
    Restarted { job_id: u64 },

    /// Job stopped
    Stopped { job_id: u64 },

    /// Error occurred
    Error { message: String },
}

/// Response for metrics (GET)
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MetricsResponse {
    Success(ExecutorSnapshot),
    Error { error: String },
}
