
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::ratelimiter::RateLimiter;


/// Action that a VirtualUser can execute
///
/// Implement this trait to support different types of actions:
/// - HTTP requests (via HttpAction)
/// - gRPC calls
/// - WebSocket messages
/// - Database queries
/// - File operations
/// - Custom business logic
pub trait Action: Send + Sync + Clone + 'static {
    /// The metrics type returned by this action
    type Metrics: ActionMetrics + Send + Sync + 'static;

    /// Execute the action and return metrics
    fn execute(&self, vuser_id: u16) -> impl std::future::Future<Output = Self::Metrics> + Send;
}

/// Metrics returned by an action execution
///
/// Different action types can return different metrics:
/// - HTTP: status code, response time, body size
/// - gRPC: status code, response time, message size
/// - Database: query time, rows affected
/// - Custom: domain-specific metrics
pub trait ActionMetrics: Send + Clone + std::fmt::Debug {
    /// Get the VUser ID that generated these metrics
    fn vuser_id(&self) -> u16;

    /// Get the timestamp when the action started
    fn timestamp(&self) -> Instant;

    /// Get the duration of the action
    fn duration(&self) -> std::time::Duration;

    /// Whether the action succeeded
    fn is_success(&self) -> bool;
}

/// Virtual User - dumb worker that executes actions
///
/// Generic over the action type (A) and its metrics (M).
/// - Share-nothing architecture (each VUser is independent)
/// - Sends metrics upstream to Executor
/// - Rate limiter always present (0.0 = unlimited to avoid branches)
/// - Supports parallel execution of actions
pub struct VirtualUser<A: Action> {
    id: u16,
    action: A,
    metrics_tx: mpsc::Sender<A::Metrics>,
    rate_limiter: RateLimiter,
    /// Number of parallel actions this VUser should execute (1 = sequential)
    parallel_actions: u8,
}

impl<A: Action> VirtualUser<A> {
    pub fn new(
        id: u16,
        action: A,
        metrics_tx: mpsc::Sender<A::Metrics>,
        requests_per_second: usize,
        parallel_actions: u8, // 1 = sequential
    ) -> Self {
        Self {
            id,
            action,
            metrics_tx,
            rate_limiter: RateLimiter::new(requests_per_second),
            parallel_actions,
        }
    }

    /// Execute actions in a loop until channel closes
    ///
    /// VUser keeps executing the same action until:
    /// - Metrics channel is full or closed
    /// - Executor signals to stop
    pub async fn run(&self) {
        loop {
            let mut handles = JoinSet::new();

            while handles.len() < self.parallel_actions as usize {
                let action = self.action.clone();
                let vuser_id = self.id;
                let ratelimiter = self.rate_limiter.clone();

                handles.spawn(async move {
                    ratelimiter.acquire().await;
                    action.execute(vuser_id).await
                });
            }

            while let Some(handle) = handles.join_next().await {
                if let Ok(metrics) = handle {
                    if self.metrics_tx.try_send(metrics).is_err() {
                        // Channel full or closed - stop executing
                        return;
                    }
                }
            }
        }
    }
}
