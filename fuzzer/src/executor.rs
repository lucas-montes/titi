use crate::planner::{Case, Assertion, AssertionKind, StoppingCondition, ExecutorType};
use crate::vuser::{VirtualUser, HttpClient, VUserMetrics};
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Executor manages a single test case with N virtual users
/// - Spawns and manages VUser tasks
/// - Enforces case assertions (p95, success rate, etc.)
/// - Checks stopping conditions
/// - Collects and sends metrics to Scheduler
pub struct Executor {
    case: Case,
    http_client: Arc<HttpClient>,
    metrics_collector: MetricsCollector,
    vuser_handles: Vec<tokio::task::JoinHandle<()>>,
    metrics_tx: mpsc::Sender<ExecutorSnapshot>,
}

/// Collects metrics from VUsers and aggregates them
pub struct MetricsCollector {
    metrics_rx: mpsc::Receiver<VUserMetrics>,
    metrics_tx: mpsc::Sender<VUserMetrics>,
    aggregated: Arc<RwLock<AggregatedMetrics>>,
    response_times: Arc<RwLock<VecDeque<Duration>>>,
    max_response_times: usize,
}

/// Aggregated metrics across all VUsers
#[derive(Debug, Default)]
pub struct AggregatedMetrics {
    pub total_requests: usize,
    pub successful: usize,
    pub failed: usize,
    pub consecutive_errors: usize,
    pub current_rps: f64,
    pub last_rps_update: Instant,
    pub status_codes: std::collections::HashMap<u16, usize>,
}

/// Snapshot sent to Scheduler
#[derive(Debug, Clone)]
pub struct ExecutorSnapshot {
    pub case_id: String,
    pub timestamp: Instant,
    pub total_requests: usize,
    pub successful: usize,
    pub failed: usize,
    pub current_rps: f64,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub active_vusers: usize,
}

impl Executor {
    pub fn new(
        case: Case,
        http_client: Arc<HttpClient>,
        metrics_tx: mpsc::Sender<ExecutorSnapshot>,
    ) -> Self {
        let (vuser_metrics_tx, vuser_metrics_rx) = mpsc::channel(10_000);

        let metrics_collector = MetricsCollector {
            metrics_rx: vuser_metrics_rx,
            metrics_tx: vuser_metrics_tx.clone(),
            aggregated: Arc::new(RwLock::new(AggregatedMetrics {
                last_rps_update: Instant::now(),
                ..Default::default()
            })),
            response_times: Arc::new(RwLock::new(VecDeque::new())),
            max_response_times: 10_000, // Keep last 10k requests for percentiles
        };

        Self {
            case,
            http_client,
            metrics_collector,
            vuser_handles: Vec::new(),
            metrics_tx,
        }
    }

    /// Start executing the test case
    pub async fn run(&mut self) -> Result<(), String> {
        // Spawn VUsers based on executor type
        let num_vusers = self.get_initial_vuser_count();
        for i in 0..num_vusers {
            self.spawn_vuser(i).await;
        }

        // Start metrics collection task
        self.start_metrics_collector().await;

        // Start validation loop
        self.validation_loop().await
    }

    fn get_initial_vuser_count(&self) -> usize {
        match &self.case.executor_type {
            ExecutorType::SingleShot => 1,
            ExecutorType::ConstantVus { vus, .. } => *vus,
            ExecutorType::RampingVus { stages } => {
                stages.first().map(|s| s.target_vus).unwrap_or(1)
            }
            ExecutorType::ConstantArrivalRate { max_vus, .. } => *max_vus,
        }
    }

    async fn spawn_vuser(&mut self, vuser_id: usize) {
        let case = self.case.clone();
        let http_client = Arc::clone(&self.http_client);
        let metrics_tx = self.metrics_collector.metrics_tx.clone();

        let handle = tokio::spawn(async move {
            let mut vuser = VirtualUser::new(vuser_id, http_client, metrics_tx);
            vuser.execute_case(case).await;
        });

        self.vuser_handles.push(handle);
    }

    async fn start_metrics_collector(&self) {
        let aggregated = Arc::clone(&self.metrics_collector.aggregated);
        let response_times = Arc::clone(&self.metrics_collector.response_times);
        let max_response_times = self.metrics_collector.max_response_times;
        let metrics_tx = self.metrics_tx.clone();
        let case_id = self.case.id.clone();

        // This would be a separate task that aggregates metrics
        // For now, just a placeholder
        tokio::spawn(async move {
            // Aggregate VUser metrics and send snapshots to Scheduler
        });
    }

    async fn validation_loop(&self) -> Result<(), String> {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Check assertions
            let aggregated = self.metrics_collector.aggregated.read().await;
            let response_times = self.metrics_collector.response_times.read().await;

            for assertion in &self.case.assertions {
                if !self.check_assertion(assertion, &aggregated, &response_times).await {
                    return Err(format!("Assertion failed: {:?}", assertion));
                }
            }

            // Check stopping conditions
            for condition in &self.case.stopping_conditions {
                if self.should_stop(condition, &aggregated, &response_times).await {
                    return Ok(());
                }
            }
        }
    }

    async fn check_assertion(
        &self,
        assertion: &Assertion,
        aggregated: &AggregatedMetrics,
        response_times: &VecDeque<Duration>,
    ) -> bool {
        match &assertion.kind {
            AssertionKind::StatusCode(expected) => {
                // Check most common status code
                aggregated.status_codes.iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(code, _)| code == expected)
                    .unwrap_or(false)
            }
            AssertionKind::P95ResponseTime(max) => {
                calculate_percentile(response_times, 0.95) <= *max
            }
            AssertionKind::SuccessRate(min) => {
                let total = aggregated.total_requests as f64;
                if total == 0.0 { return true; }
                (aggregated.successful as f64 / total) >= *min
            }
            AssertionKind::ErrorRate(max) => {
                let total = aggregated.total_requests as f64;
                if total == 0.0 { return true; }
                (aggregated.failed as f64 / total) <= *max
            }
            AssertionKind::MinRps(min) => {
                aggregated.current_rps >= *min
            }
            AssertionKind::MaxRps(max) => {
                aggregated.current_rps <= *max
            }
        }
    }

    async fn should_stop(
        &self,
        condition: &StoppingCondition,
        aggregated: &AggregatedMetrics,
        response_times: &VecDeque<Duration>,
    ) -> bool {
        match condition {
            StoppingCondition::FailureRate(threshold) => {
                let total = aggregated.total_requests as f64;
                if total == 0.0 { return false; }
                (aggregated.failed as f64 / total) >= *threshold
            }
            StoppingCondition::ConsecutiveErrors(max) => {
                aggregated.consecutive_errors >= *max
            }
            StoppingCondition::P95ResponseTime(max) => {
                calculate_percentile(response_times, 0.95) > *max
            }
            StoppingCondition::TotalRequests(max) => {
                aggregated.total_requests >= *max
            }
        }
    }
}

/// Calculate percentile from response times
fn calculate_percentile(times: &VecDeque<Duration>, percentile: f64) -> Duration {
    if times.is_empty() {
        return Duration::from_secs(0);
    }

    let mut sorted: Vec<Duration> = times.iter().copied().collect();
    sorted.sort();

    let index = ((sorted.len() as f64) * percentile) as usize;
    sorted.get(index).copied().unwrap_or(Duration::from_secs(0))
}
