use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use reqwest::Client as ReqwestClient;
use crate::planner::{Case, RequestTemplate, Method};

/// Virtual User - dumb worker that executes requests from a case
/// - Share-nothing architecture (each VUser is independent)
/// - Shares HttpClient via Arc (reqwest uses connection pooling internally)
/// - Sends metrics upstream to Executor
/// - Can have per-VUser rate limiting
pub struct VirtualUser {
    pub id: usize,
    http_client: Arc<HttpClient>,
    metrics_tx: mpsc::Sender<VUserMetrics>,
    rate_limiter: Option<RateLimiter>,
}

/// HTTP Client wrapper (shared via Arc across VUsers)
pub struct HttpClient {
    client: ReqwestClient,
}

/// Per-VUser rate limiter
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
        http_client: Arc<HttpClient>,
        metrics_tx: mpsc::Sender<VUserMetrics>,
    ) -> Self {
        Self {
            id,
            http_client,
            metrics_tx,
            rate_limiter: None,
        }
    }

    pub fn with_rate_limit(mut self, rps: f64) -> Self {
        self.rate_limiter = Some(RateLimiter::new(rps));
        self
    }

    /// Execute the test case
    /// VUser keeps executing requests from the template until stopped
    pub async fn execute_case(&mut self, case: Case) {
        let template = &case.request_template;

        loop {
            // Apply rate limiting if configured
            if let Some(limiter) = &mut self.rate_limiter {
                limiter.wait().await;
            }

            // Execute request and collect metrics
            let metrics = self.execute_request(template).await;

            // Send metrics upstream (non-blocking)
            if self.metrics_tx.try_send(metrics).is_err() {
                // Channel full or closed - stop executing
                break;
            }
        }
    }

    async fn execute_request(&self, template: &RequestTemplate) -> VUserMetrics {
        let start = Instant::now();

        // Build request
        let request = match template.method {
            Method::Get => self.http_client.client.get(&template.url),
            Method::Post => self.http_client.client.post(&template.url),
            Method::Put => self.http_client.client.put(&template.url),
            Method::Delete => self.http_client.client.delete(&template.url),
            Method::Patch => self.http_client.client.patch(&template.url),
        };

        // Add headers
        let mut request = request;
        for (key, value) in &template.headers {
            request = request.header(key, value);
        }

        // Add body if present
        if let Some(body) = &template.body {
            request = request.body(body.clone());
        }

        // Execute
        let result = request.send().await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                let status = response.status().as_u16();
                let success = response.status().is_success();

                VUserMetrics {
                    vuser_id: self.id,
                    timestamp: start,
                    status_code: Some(status),
                    response_time: elapsed,
                    success,
                    error: None,
                }
            }
            Err(err) => {
                VUserMetrics {
                    vuser_id: self.id,
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

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::builder()
                .pool_max_idle_per_host(100)
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
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
