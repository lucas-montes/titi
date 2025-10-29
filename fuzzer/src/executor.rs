use crate::metrics::MetricsCollector;
use crate::planner::{TestScenario, TestCase, PlannerFeedback, ErrorCategory, CaseIterator};
use crate::vuser::{Action, ActionMetrics};
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
#[derive(Debug, Clone)]
pub struct ExecutorSnapshot {
    pub scenario_id: String,
    pub timestamp: Instant,
    pub elapsed: Duration,

    /// Current case being executed (if any)
    pub current_case_id: Option<String>,

    /// Number of cases completed
    pub cases_completed: usize,

    /// Success rate across all cases in this scenario
    pub success_rate: f64,

    /// Total actions executed
    pub total_actions: usize,

    /// Error distribution
    pub error_counts: HashMap<ErrorCategory, usize>,

    /// Current throughput (actions/sec)
    pub throughput: f64,

    /// Whether this executor has finished
    pub is_finished: bool,
}

/// Result of executing a scenario
pub type ExecutionResult<M> = Result<ExecutionSummary<M>, ExecutionError>;

/// Summary of a completed scenario execution
#[derive(Debug)]
pub struct ExecutionSummary<M: ActionMetrics> {
    pub scenario_id: String,
    pub duration: Duration,
    pub cases_executed: usize,
    pub total_actions: usize,
    pub success_rate: f64,
    pub metrics: Vec<M>,
    pub feedback: PlannerFeedback,
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

// ============================================================================
// OLD ARCHITECTURE - To be phased out
// ============================================================================

use tokio::sync::{mpsc as old_mpsc};

/// Executor manages a single test case with N virtual users
/// - Spawns and manages VUser tasks
/// - Enforces case assertions (p95, success rate, etc.)
/// - Checks stopping conditions
/// - Collects and sends metrics to Scheduler
pub struct OldExecutor {
    /// The case received with the tests cases to execute and the validations to apply
    case: OldCase,
    /// The metrics from the vusers
    metrics_collector: OldMetricsCollector,
    /// Handles for spawned VUser tasks
    vuser_handles: Vec<tokio::task::JoinHandle<()>>,
    /// Channel to send metrics to the scheduler
    metrics_tx: old_mpsc::Sender<OldExecutorSnapshot>,
    /// Channel to receive metrics from the vusers
    metrics_rx: old_mpsc::Receiver<OldExecutorSnapshot>,
}

// Placeholder types for old architecture
type OldCase = ();
type OldMetricsCollector = ();
type OldExecutorSnapshot = ();
