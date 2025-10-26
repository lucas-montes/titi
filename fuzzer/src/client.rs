use reqwest::Client as ReqwestClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::{Duration, Instant};

use crate::metrics::{MetricsAggregator, RequestMetrics};


/// Categorized request errors (zero-allocation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestError {
    /// Connection-related errors (DNS, TCP, etc.)
    Connection,
    /// Timeout occurred
    Timeout,
    /// TLS/SSL errors
    Tls,
    /// HTTP protocol errors
    Protocol,
    /// Failed to read response body
    BodyRead,
    /// Request was cancelled or aborted
    Cancelled,
    /// Other/unknown errors
    Other,
}

impl From<reqwest::Error> for RequestError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else if err.is_connect() {
            Self::Connection
        } else if err.is_request() {
            Self::Protocol
        } else if err.is_body() {
            Self::BodyRead
        } else {
            Self::Other
        }
    }
}


impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Connection => "connection",
            Self::Timeout => "timeout",
            Self::Tls => "tls",
            Self::Protocol => "protocol",
            Self::BodyRead => "body_read",
            Self::Cancelled => "cancelled",
            Self::Other => "other",
        };
        f.write_str(text)
    }
}


/// HTTP Client wrapper
struct HttpClient {
    client: ReqwestClient,
    max_concurrent: usize,
}

impl HttpClient {
    /// Execute a single request and return generic metrics
    async fn execute<M: RequestMetrics>(&self, url: &str) -> M {
        let start = Instant::now();
        let response = self.client.get(url).send().await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match resp.bytes().await {
                    Ok(bytes) => M::success(start.elapsed(), status, bytes.len()),
                    Err(e) => M::failure(start.elapsed(), e),
                }
            }
            Err(e) => M::failure(start.elapsed(), e),
        }
    }
}


/// Metrics collector that aggregates results
struct MetricsCollector<A: MetricsAggregator> {
    receiver: mpsc::Receiver<A::Request>,
}

impl<A: MetricsAggregator> MetricsCollector<A> {
    async fn collect(mut self) -> A {
        let mut aggregator = A::default();

        while let Some(metric) = self.receiver.recv().await {
            aggregator.add(metric);
        }

        aggregator.finalize();
        aggregator
    }
}

/// Generic test runner with pluggable metrics
struct TestRunner<A: MetricsAggregator> {
    client: Arc<HttpClient>,
    metrics_sender: mpsc::Sender<A::Request>,
    metrics_collector: Option<MetricsCollector<A>>,
}

impl<A: MetricsAggregator> TestRunner<A> {
    /// Execute load test
    async fn run(&self, url: &str, duration: Duration) {
        let url = url.to_string();
        let client = Arc::clone(&self.client);
        let sender = self.metrics_sender.clone();
        let max_concurrent = client.max_concurrent;

        let test_handle = tokio::spawn(async move {
            let end_time = Instant::now() + duration;
            let mut tasks = JoinSet::new();

            while Instant::now() < end_time {
                // Backpressure control
                while tasks.len() >= max_concurrent {
                    if let Some(result) = tasks.join_next().await {
                        if let Ok(metrics) = result {
                            let _ = sender.send(metrics).await;
                        }
                    }
                }

                // Spawn new request
                let client_clone = Arc::clone(&client);
                let url_clone = url.clone();
                tasks.spawn(async move {
                    client_clone.execute::<A::Request>(&url_clone).await
                });
            }

            // Drain remaining tasks
            while let Some(result) = tasks.join_next().await {
                if let Ok(metrics) = result {
                    let _ = sender.send(metrics).await;
                }
            }
        });

        let _ = test_handle.await;
    }

    /// Finish the test and collect metrics
    async fn finish(mut self) -> A {
        // Drop sender to signal completion
        drop(self.metrics_sender);

        // Collect metrics
        if let Some(collector) = self.metrics_collector.take() {
            collector.collect().await
        } else {
            A::default()
        }
    }
}

/// Builder for creating test runners
struct TestRunnerBuilder {
    max_concurrent: usize,
    connect_timeout: Duration,
    user_agent: Option<String>,
    verify_ssl: bool,
}

impl TestRunnerBuilder {
    fn new() -> Self {
        Self {
            max_concurrent: 100,
            connect_timeout: Duration::from_secs(10),
            user_agent: Some("Fuzzer/1.0".to_string()),
            verify_ssl: true,
        }
    }

    fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    fn verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = verify;
        self
    }

    /// Build a test runner with specified metrics aggregator type
    fn build<A: MetricsAggregator>(self) -> Result<TestRunner<A>, String> {
        let mut builder = ReqwestClient::builder()
            .connect_timeout(self.connect_timeout)
            .pool_max_idle_per_host(100)
            .tcp_nodelay(true)
            .danger_accept_invalid_certs(!self.verify_ssl);

        if let Some(ua) = self.user_agent {
            builder = builder.user_agent(ua);
        }

        let client = builder
            .build()
            .map_err(|e| format!("Failed to build client: {}", e))?;

        let http_client = Arc::new(HttpClient {
            client,
            max_concurrent: self.max_concurrent,
        });

        let (tx, rx) = mpsc::channel(10000);
        let collector = MetricsCollector { receiver: rx };

        Ok(TestRunner {
            client: http_client,
            metrics_sender: tx,
            metrics_collector: Some(collector),
        })
    }
}

impl Default for TestRunnerBuilder {
    fn default() -> Self {
        Self::new()
    }
}


/// Test configuration that bridges config and runner
struct TestConfig {
    target_url: String,
    max_concurrent: usize,
    timeout: Duration,
    test_duration: Duration,
    user_agent: Option<String>,
    verify_ssl: bool,
}

impl From<&crate::config::TestScenario> for TestConfig {
    fn from(scenario: &crate::config::TestScenario) -> Self {
        let exec = scenario.execution();
        let target = scenario.target();
        let advanced = scenario.advanced();

        Self {
            target_url: target.base_url.clone(),
            max_concurrent: exec.concurrent_requests,
            timeout: Duration::from_secs(exec.timeout_seconds),
            test_duration: Duration::from_secs(exec.max_duration_seconds.unwrap_or(60)),
            user_agent: advanced.user_agent().map(|s| s.to_string()),
            verify_ssl: advanced.verify_ssl(),
        }
    }
}

impl TestConfig {
    /// Build a test runner with the specified metrics type
    fn build<A: MetricsAggregator>(&self) -> Result<TestRunner<A>, String> {
        let mut builder = TestRunnerBuilder::new()
            .max_concurrent(self.max_concurrent)
            .connect_timeout(self.timeout)
            .verify_ssl(self.verify_ssl);

        if let Some(ua) = &self.user_agent {
            builder = builder.user_agent(ua.clone());
        }

        builder.build()
    }

    /// Get the target URL
    fn url(&self) -> &str {
        &self.target_url
    }

    /// Get the test duration
    fn duration(&self) -> Duration {
        self.test_duration
    }
}
