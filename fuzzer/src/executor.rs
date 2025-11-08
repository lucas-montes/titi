use crate::metrics::MetricsCollector;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Executor manages a single test scenario with N virtual users.
///
/// Responsibilities:
/// - Lazily generate test cases from the scenario's parameter space
/// - Spawn VUsers to execute actions
/// - Collect and aggregate metrics
/// - Check stopping conditions (per-case and per-scenario)
/// - Send periodic snapshots to Scheduler
/// - Generate feedback for Planner
///
/// Scope: One Executor per TestScenario
///
/// NOTE: Uses monomorphization - no dynamic dispatch (`Box<dyn>`).
/// The iterator type is concrete and known at compile time.
pub struct Executor<A: Action> {
    /// The scenario being executed (consumed when creating iterator)
    scenario: Option<TestScenario<A>>,

    /// Iterator for lazily generating test cases
    /// Concrete type - no Box<dyn>, full type safety and zero-cost abstraction
    case_iterator: Option<CaseIterator<A>>,

    /// Number of virtual users to spawn per case
    num_vusers: usize,

    /// Metrics collector for this executor
    metrics_collector: MetricsCollector<A::Metrics>,

    /// Channel to send snapshots to scheduler
    snapshot_tx: mpsc::Sender<ExecutorSnapshot>,

    /// Channel to receive stop signal from scheduler
    stop_rx: mpsc::Receiver<()>,

    /// Execution start time
    start_time: Instant,

    /// Statistics for feedback generation
    stats: ExecutionStats,
}

/// Execution statistics for a scenario (used for feedback)
#[derive(Debug, Default)]
struct ExecutionStats {
    total_cases: usize,
    successful_cases: usize,
    failed_cases: usize,
    consecutive_successes: usize,
    consecutive_failures: usize,
    error_counts: HashMap<ErrorCategory, usize>,
}

/// Protocol-agnostic snapshot of executor state (for metrics streaming)
#[derive(Debug, Clone, Serialize)]
pub struct ExecutorSnapshot {
    scenario_id: String,
    timestamp: u64,
    elapsed: Duration,

    /// Current case being executed (if any)
    current_case_id: Option<String>,

    /// Number of cases completed
    cases_completed: usize,

    /// Success rate across all cases in this scenario
    success_rate: f64,

    /// Total actions executed
    total_actions: usize,

    /// Error distribution
    error_counts: HashMap<ErrorCategory, usize>,

    /// Current throughput (actions/sec)
    throughput: f64,

    /// Whether this executor has finished
    is_finished: bool,
}

/// Result of executing a scenario
pub type ExecutionResult<M> = Result<ExecutionSummary<M>, ExecutionError>;

/// Summary of a completed scenario execution
#[derive(Debug)]
pub struct ExecutionSummary<M: ActionMetrics> {
    scenario_id: String,
    duration: Duration,
    cases_executed: usize,
    total_actions: usize,
    success_rate: f64,
    metrics: Vec<M>,
    feedback: PlannerFeedback,
}

/// Errors that can occur during execution
#[derive(Debug)]
pub enum ExecutionError {
    ScenarioFailed(String),
    StoppedByScheduler,
    Timeout,
}

impl<A: Action> Executor<A> {
    /// Create a new executor for a scenario
    pub fn new(
        _scenario: TestScenario<A>,
        _num_vusers: usize,
        _snapshot_tx: mpsc::Sender<ExecutorSnapshot>,
        _stop_rx: mpsc::Receiver<()>,
    ) -> Self {
        // Implementation details omitted
        todo!("Executor::new")
    }

    /// Run the executor until completion or stopped
    pub async fn run(self) -> ExecutionResult<A::Metrics> {
        // Implementation details omitted
        todo!("Executor::run")
    }

    /// Generate feedback for the planner based on execution results
    fn generate_feedback(&self) -> PlannerFeedback {
        // Implementation details omitted
        todo!("Executor::generate_feedback")
    }

    /// Check if scenario stopping conditions are met
    fn should_stop(&self) -> bool {
        // Implementation details omitted
        todo!("Executor::should_stop")
    }

    /// Send a snapshot to the scheduler
    async fn send_snapshot(&self) {
        // Implementation details omitted
        todo!("Executor::send_snapshot")
    }
}
