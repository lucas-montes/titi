use crate::planner::TestPlanner;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use std::time::Instant;

/// Scheduler orchestrates test execution
/// - Polls cases from Planner
/// - Creates Executors for each case
/// - Manages MetricsHub (streaming + recording)
/// - Provides feedback to adaptive planners
pub struct Scheduler<P: TestPlanner> {
    planner: P,
    metrics_hub: Arc<MetricsHub>,
    config: SchedulerConfig,
}

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub enable_streaming: bool,
    pub enable_recording: bool,
    pub storage: StorageConfig,
    pub parallel_execution: bool,
}

#[derive(Debug, Clone)]
pub enum StorageConfig {
    Database(String),
    File(String),
    Memory,
    None,
}

/// Central metrics hub - coordinates streaming and recording
pub struct MetricsHub {
    snapshot_tx: mpsc::Sender<ExecutorSnapshot>,
    snapshot_rx: Option<mpsc::Receiver<ExecutorSnapshot>>,
    stream: MetricsStream,
    recorder: MetricsRecorder,
    global_metrics: Arc<RwLock<GlobalMetrics>>,
}

/// Real-time streaming to WebSocket clients
pub struct MetricsStream {
    ws_clients: Arc<RwLock<Vec<mpsc::Sender<GlobalSnapshot>>>>,
    recent_history: Arc<RwLock<Vec<GlobalSnapshot>>>,
    max_history_size: usize,
}

/// Persistent storage of metrics
pub struct MetricsRecorder {
    storage: StorageConfig,
    buffer: Arc<RwLock<Vec<ExecutorSnapshot>>>,
    buffer_size: usize,
}

/// Snapshot from a single executor
#[derive(Debug, Clone)]
pub struct ExecutorSnapshot {
    pub case_id: String,
    pub timestamp: Instant,
    pub total_requests: usize,
    pub successful: usize,
    pub failed: usize,
    pub current_rps: f64,
    pub p50: std::time::Duration,
    pub p95: std::time::Duration,
    pub p99: std::time::Duration,
    pub active_vusers: usize,
}

/// Global snapshot across all executors
#[derive(Debug, Clone)]
pub struct GlobalSnapshot {
    pub timestamp: Instant,
    pub executors: Vec<ExecutorSnapshot>,
    pub total_requests: usize,
    pub total_successful: usize,
    pub total_failed: usize,
    pub overall_rps: f64,
}

/// In-memory global metrics aggregation
#[derive(Debug, Default)]
pub struct GlobalMetrics {
    snapshots: Vec<ExecutorSnapshot>,
    total_requests: usize,
    total_successful: usize,
    total_failed: usize,
}

impl<P: TestPlanner> Scheduler<P> {
    pub fn new(planner: P, config: SchedulerConfig) -> Self {
        let (snapshot_tx, snapshot_rx) = mpsc::channel(10_000);

        let metrics_hub = Arc::new(MetricsHub {
            snapshot_tx,
            snapshot_rx: Some(snapshot_rx),
            stream: MetricsStream::new(),
            recorder: MetricsRecorder::new(config.storage.clone()),
            global_metrics: Arc::new(RwLock::new(GlobalMetrics::default())),
        });

        Self {
            planner,
            metrics_hub,
            config,
        }
    }

    pub fn snapshot_sender(&self) -> mpsc::Sender<ExecutorSnapshot> {
        self.metrics_hub.snapshot_tx.clone()
    }
}

impl MetricsStream {
    fn new() -> Self {
        Self {
            ws_clients: Arc::new(RwLock::new(Vec::new())),
            recent_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 120, // Last 60s at 2 snapshots/sec
        }
    }
}

impl MetricsRecorder {
    fn new(storage: StorageConfig) -> Self {
        Self {
            storage,
            buffer: Arc::new(RwLock::new(Vec::new())),
            buffer_size: 100,
        }
    }
}
