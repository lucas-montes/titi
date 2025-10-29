use crate::planner::ErrorCategory;
use crate::vuser::ActionMetrics;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

// ============================================================================
// NEW ARCHITECTURE - Generic MetricsCollector<M: ActionMetrics>
// ============================================================================

/// Collects and aggregates metrics from VUsers.
///
/// Supports two modes:
/// 1. Streaming: Update aggregates as metrics arrive (for real-time monitoring)
/// 2. Batch: Store raw metrics for lazy percentile calculation (for final reports)
pub struct MetricsCollector<M: ActionMetrics> {
    /// Channel to receive metrics from VUsers
    metrics_rx: mpsc::Receiver<M>,

    /// All raw metrics (for batch operations like percentiles)
    raw_metrics: Vec<M>,

    /// Streaming aggregates (updated on every metric arrival)
    streaming_stats: StreamingStats,

    /// Error distribution
    error_counts: HashMap<ErrorCategory, usize>,

    /// Start time of collection
    start_time: Instant,
}

/// Streaming statistics (updated incrementally)
#[derive(Debug, Default, Clone)]
pub struct StreamingStats {
    pub total_count: usize,
    pub success_count: usize,
    pub failure_count: usize,

    /// Running sum of durations (for mean calculation)
    pub duration_sum: Duration,

    /// Min/max durations seen so far
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
}

impl<M: ActionMetrics> MetricsCollector<M> {
    /// Create a new metrics collector
    pub fn new(_metrics_rx: mpsc::Receiver<M>) -> Self {
        // Implementation details omitted
        todo!("MetricsCollector::new")
    }

    /// Process incoming metrics (non-blocking, processes all available)
    pub fn process_incoming(&mut self) {
        // Implementation details omitted
        todo!("MetricsCollector::process_incoming")
    }

    /// Get current streaming statistics (fast, no sorting)
    pub fn get_streaming_stats(&self) -> &StreamingStats {
        &self.streaming_stats
    }

    /// Calculate percentile (lazy, requires sorting)
    pub fn calculate_percentile(&mut self, _p: f64) -> Option<Duration> {
        // Implementation details omitted
        todo!("MetricsCollector::calculate_percentile")
    }

    /// Get error distribution
    pub fn get_error_counts(&self) -> &HashMap<ErrorCategory, usize> {
        &self.error_counts
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.streaming_stats.total_count == 0 {
            return 1.0;
        }
        self.streaming_stats.success_count as f64 / self.streaming_stats.total_count as f64
    }

    /// Get throughput (actions per second)
    pub fn throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed();
        if elapsed.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.streaming_stats.total_count as f64 / elapsed.as_secs_f64()
    }

    /// Consume collector and return all raw metrics
    pub fn into_metrics(self) -> Vec<M> {
        self.raw_metrics
    }
}

// ============================================================================
// OLD ARCHITECTURE - To be phased out
// ============================================================================

use crate::client::RequestError;

/// Trait for collecting metrics from individual requests
pub trait RequestMetrics: Send + 'static {
    /// Record a successful request
    fn success(duration: Duration, status_code: u16, bytes_received: usize) -> Self;

    /// Record a failed request
    fn failure(duration: Duration, error: impl Into<RequestError>) -> Self;
}

/// Trait for aggregating metrics across multiple requests
pub trait MetricsAggregator: Default {
    type Request: RequestMetrics;

    /// Add a single request metric to the aggregation
    fn add(&mut self, metric: Self::Request);

    /// Finalize the aggregation (e.g., sort for percentiles)
    fn finalize(&mut self);
}
