use crate::executor::{ExecutorSnapshot, ExecutionResult};
use crate::planner::{TestPlanner, PlannerFeedback, TestScenario};
use crate::vuser::Action;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// ============================================================================
// NEW ARCHITECTURE - Generic Scheduler<P: TestPlanner>
// ============================================================================

/// Scheduler orchestrates test execution across multiple scenarios.
///
/// Responsibilities:
/// - Pull test scenarios from Planner
/// - Spawn Executors (one per scenario)
/// - Collect snapshots from all executors
/// - Aggregate metrics across scenarios
/// - Send feedback to Planner for adaptive test generation
/// - Manage MetricsHub (streaming via WebSocket + persistent recording)
/// - Check global stopping conditions
pub struct Scheduler<P: TestPlanner> {
    /// Test planner that generates scenarios
    planner: P,

    /// Configuration
    config: SchedulerConfig,

    /// Channel to receive snapshots from executors
    snapshot_rx: mpsc::Receiver<ExecutorSnapshot>,

    /// Channel to send snapshots to MetricsHub
    hub_tx: mpsc::Sender<ExecutorSnapshot>,

    /// Running executors (scenario_id -> join handle)
    executors: HashMap<String, JoinHandle<ExecutionResult<<P::Action as Action>::Metrics>>>,

    /// Global statistics
    global_stats: GlobalStats,

    /// Start time
    start_time: Instant,
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum number of concurrent executors (scenarios running in parallel)
    pub max_concurrent_scenarios: usize,

    /// Number of VUsers per executor
    pub vusers_per_scenario: usize,

    /// How often to check for stopping conditions
    pub check_interval: Duration,

    /// Global stopping conditions
    pub global_stopping_conditions: GlobalStoppingConditions,
}

/// Global stopping conditions (across all scenarios)
#[derive(Debug, Clone, Copy)]
pub struct GlobalStoppingConditions {
    /// Stop after total duration
    pub max_duration: Option<Duration>,

    /// Stop after total number of scenarios
    pub max_scenarios: Option<usize>,

    /// Stop after total number of actions
    pub max_total_actions: Option<usize>,
}

/// Global statistics across all scenarios
#[derive(Debug, Default)]
struct GlobalStats {
    scenarios_completed: usize,
    scenarios_failed: usize,
    total_actions: usize,
    total_errors: usize,
}

/// Result of running the scheduler
pub type SchedulerResult = Result<SchedulerSummary, SchedulerError>;

/// Summary of completed scheduler run
#[derive(Debug)]
pub struct SchedulerSummary {
    pub duration: Duration,
    pub scenarios_completed: usize,
    pub scenarios_failed: usize,
    pub total_actions: usize,
    pub overall_success_rate: f64,
}

/// Errors that can occur in the scheduler
#[derive(Debug)]
pub enum SchedulerError {
    ExecutorFailed(String),
    GlobalTimeout,
    StoppedByUser,
}

impl<P: TestPlanner> Scheduler<P> {
    /// Create a new scheduler
    pub fn new(
        _planner: P,
        _config: SchedulerConfig,
        _hub_tx: mpsc::Sender<ExecutorSnapshot>,
    ) -> Self {
        // Implementation details omitted
        todo!("Scheduler::new")
    }

    /// Run the scheduler until completion or stopped
    pub async fn run(self) -> SchedulerResult {
        // Implementation details omitted
        todo!("Scheduler::run")
    }

    /// Spawn an executor for a scenario
    async fn spawn_executor(&mut self, _scenario: TestScenario<P::Action>) {
        // Implementation details omitted
        todo!("Scheduler::spawn_executor")
    }

    /// Process snapshots from executors
    async fn process_snapshots(&mut self) {
        // Implementation details omitted
        todo!("Scheduler::process_snapshots")
    }

    /// Send feedback to planner based on executor results
    fn send_feedback(&mut self, _feedback: PlannerFeedback) {
        // Implementation details omitted
        todo!("Scheduler::send_feedback")
    }

    /// Check if global stopping conditions are met
    fn should_stop(&self) -> bool {
        // Implementation details omitted
        todo!("Scheduler::should_stop")
    }
}

// ============================================================================
// OLD ARCHITECTURE - To be phased out
// ============================================================================

// trait HubMetricsAggregator {
//     fn add_snapshot(&mut self, snapshot: OldExecutorSnapshot);
//     fn aggregate(&self) -> GlobalSnapshot;
// }
//
// type OldExecutorSnapshot = ();
// type GlobalSnapshot = ();
//
// pub struct OldScheduler<P: TestPlanner, Metrics: HubMetricsAggregator> {
//     planner: P,
//     metrics: Metrics,
//     executors_rx: mpsc::Receiver<OldExecutorSnapshot>,
//     matrics_exporter: mpsc::Sender<OldExecutorSnapshot>,
// }
