use std::time::Instant;
use tokio::sync::mpsc;
use reqwest::Client as ReqwestClient;
use crate::planner::{Case, RequestTemplate, Method};

/// Virtual User - dumb worker that executes requests from a case
/// - Share-nothing architecture (each VUser is independent)
/// - Shares HttpClient (reqwest::Client already uses Arc internally for connection pooling)
/// - Sends metrics upstream to Executor
/// - All VUsers have rate limiter (0.0 = unlimited to avoid branches)
pub struct VirtualUser {
    pub id: usize,
    http_client: ReqwestClient,
    metrics_tx: mpsc::Sender<VUserMetrics>,
    rate_limiter: RateLimiter,
    /// Number of parallel requests this VUser should execute (1 = sequential)
    parallel_requests: usize,
}

/// Per-VUser rate limiter
/// Uses 0.0 for unlimited to avoid Option and branching
pub struct RateLimiter {
    requests_per_second: f64,
    last_request: Option<Instant>,
}

/// Metrics from a single request
#[derive(Debug, Clone)]
pub struct VUserMetrics {
    pub vuser_id: usize,
    pub timestamp: Instant,
    pub status_code: Option<u16>,
    pub response_time: std::time::Duration,
    pub success: bool,
    pub error: Option<String>,
}

impl VirtualUser {
    pub fn new(
        id: usize,
        http_client: ReqwestClient,
        metrics_tx: mpsc::Sender<VUserMetrics>,
        requests_per_second: f64, // 0.0 = unlimited
        parallel_requests: usize,  // 1 = sequential
    ) -> Self {
        Self {
            id,
            http_client,
            metrics_tx,
            rate_limiter: RateLimiter::new(requests_per_second),
            parallel_requests,
        }
    }

    /// Execute the test case
    /// VUser keeps executing requests from the template until stopped
    pub async fn execute_case(&mut self, case: Case) {
        let template = &case.request_template;

        loop {
            // Apply rate limiting (0.0 = unlimited, no branch needed)
            self.rate_limiter.wait().await;

            if self.parallel_requests == 1 {
                // Sequential execution (most common case)
                let metrics = self.execute_request(template).await;
                if self.metrics_tx.try_send(metrics).is_err() {
                    break;
                }
            } else {
                // Parallel execution: spawn N requests concurrently
                let mut handles = Vec::with_capacity(self.parallel_requests);

                for _ in 0..self.parallel_requests {
                    let client = self.http_client.clone();
                    let template = template.clone();
                    let vuser_id = self.id;

                    handles.push(tokio::spawn(async move {
                        Self::execute_request_static(vuser_id, &client, &template).await
                    }));
                }

                // Wait for all requests to complete
                for handle in handles {
                    if let Ok(metrics) = handle.await {
                        if self.metrics_tx.try_send(metrics).is_err() {
                            // Channel full or closed - stop executing
                            return;
                        }
                    }
                }
            }
        }
    }

    async fn execute_request(&self, template: &RequestTemplate) -> VUserMetrics {
        Self::execute_request_static(self.id, &self.http_client, template).await
    }

    /// Static version for parallel execution
    async fn execute_request_static(
        vuser_id: usize,
        client: &ReqwestClient,
        template: &RequestTemplate,
    ) -> VUserMetrics {
        let start = Instant::now();

        // Build request using From trait
        let request_builder = template.to_request_builder(client);

        // Execute
        let result = request_builder.send().await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                let status = response.status().as_u16();
                let success = response.status().is_success();

                VUserMetrics {
                    vuser_id,
                    timestamp: start,
                    status_code: Some(status),
                    response_time: elapsed,
                    success,
                    error: None,
                }
            }
            Err(err) => {
                VUserMetrics {
                    vuser_id,
                    timestamp: start,
                    status_code: None,
                    response_time: elapsed,
                    success: false,
                    error: Some(err.to_string()),
                }
            }
        }
    }
}

impl RateLimiter {
    fn new(requests_per_second: f64) -> Self {
        Self {
            requests_per_second,
            last_request: None,
        }
    }

    async fn wait(&mut self) {
        // 0.0 = unlimited, skip waiting entirely (branch predictor friendly)
        if self.requests_per_second <= 0.0 {
            return;
        }

        if let Some(last) = self.last_request {
            let min_interval = std::time::Duration::from_secs_f64(1.0 / self.requests_per_second);
            let elapsed = last.elapsed();

            if elapsed < min_interval {
                tokio::time::sleep(min_interval - elapsed).await;
            }
        }

        self.last_request = Some(Instant::now());
    }
}

/// Helper to create HTTP client with optimal settings
pub fn create_http_client() -> ReqwestClient {
    ReqwestClient::builder()
        .pool_max_idle_per_host(100)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}
