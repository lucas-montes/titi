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


/// Basic metrics for a single request
#[derive(Debug, Clone)]
struct BasicRequestMetrics {
    duration: Duration,
    status_code: Option<u16>,
    bytes_received: usize,
    error: Option<RequestError>,
}

impl RequestMetrics for BasicRequestMetrics {
    fn success(duration: Duration, status_code: u16, bytes_received: usize) -> Self {
        Self {
            duration,
            status_code: Some(status_code),
            bytes_received,
            error: None,
        }
    }

    fn failure(duration: Duration, error: impl Into<RequestError>) -> Self {
        Self {
            duration,
            status_code: None,
            bytes_received: 0,
            error: Some(error.into()),
        }
    }
}


/// Basic aggregated metrics with percentile calculations
#[derive(Debug, Default)]
struct BasicAggregatedMetrics {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    total_bytes_received: usize,
    durations: Vec<Duration>,
}

impl MetricsAggregator for BasicAggregatedMetrics {
    type Request = BasicRequestMetrics;

    fn add(&mut self, metric: Self::Request) {
        self.total_requests += 1;
        self.durations.push(metric.duration);
        self.total_bytes_received += metric.bytes_received;

        if metric.error.is_none() {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
    }

    fn finalize(&mut self) {
        self.durations.sort();
    }
}

impl BasicAggregatedMetrics {
    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    fn avg_duration(&self) -> Duration {
        if self.durations.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.durations.iter().sum();
            total / self.durations.len() as u32
        }
    }

    fn p50(&self) -> Duration {
        self.percentile(0.50)
    }

    fn p95(&self) -> Duration {
        self.percentile(0.95)
    }

    fn p99(&self) -> Duration {
        self.percentile(0.99)
    }

    fn percentile(&self, p: f64) -> Duration {
        if self.durations.is_empty() {
            return Duration::ZERO;
        }
        let index = ((self.durations.len() as f64) * p).ceil() as usize - 1;
        self.durations[index.min(self.durations.len() - 1)]
    }
}


/// Detailed metrics including timestamps
#[derive(Debug, Clone)]
struct DetailedRequestMetrics {
    duration: Duration,
    status_code: Option<u16>,
    bytes_received: usize,
    error: Option<RequestError>,
    timestamp: Instant,
}

impl RequestMetrics for DetailedRequestMetrics {
    fn success(duration: Duration, status_code: u16, bytes_received: usize) -> Self {
        Self {
            duration,
            status_code: Some(status_code),
            bytes_received,
            error: None,
            timestamp: Instant::now(),
        }
    }

    fn failure(duration: Duration, error: impl Into<RequestError>) -> Self {
        Self {
            duration,
            status_code: None,
            bytes_received: 0,
            error: Some(error.into()),
            timestamp: Instant::now(),
        }
    }
}

/// Detailed aggregated metrics with error categorization
#[derive(Debug, Default)]
struct DetailedAggregatedMetrics {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    total_bytes_received: usize,
    status_codes: std::collections::HashMap<u16, usize>,
    errors: std::collections::HashMap<RequestError, usize>,
    durations: Vec<Duration>,
}

impl MetricsAggregator for DetailedAggregatedMetrics {
    type Request = DetailedRequestMetrics;

    fn add(&mut self, metric: Self::Request) {
        self.total_requests += 1;
        self.durations.push(metric.duration);
        self.total_bytes_received += metric.bytes_received;

        if let Some(status) = metric.status_code {
            *self.status_codes.entry(status).or_insert(0) += 1;
            self.successful_requests += 1;
        }

        if let Some(error) = metric.error {
            *self.errors.entry(error).or_insert(0) += 1;
            self.failed_requests += 1;
        }
    }

    fn finalize(&mut self) {
        self.durations.sort();
    }
}

impl DetailedAggregatedMetrics {
    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    fn avg_duration(&self) -> Duration {
        if self.durations.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.durations.iter().sum();
            total / self.durations.len() as u32
        }
    }

    fn p50(&self) -> Duration {
        self.percentile(0.50)
    }

    fn p95(&self) -> Duration {
        self.percentile(0.95)
    }

    fn p99(&self) -> Duration {
        self.percentile(0.99)
    }

    fn percentile(&self, p: f64) -> Duration {
        if self.durations.is_empty() {
            return Duration::ZERO;
        }
        let index = ((self.durations.len() as f64) * p).ceil() as usize - 1;
        self.durations[index.min(self.durations.len() - 1)]
    }
}
