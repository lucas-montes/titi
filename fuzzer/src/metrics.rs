use tokio::time::{Duration, Instant};

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
