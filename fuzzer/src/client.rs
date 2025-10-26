use reqwest::Client as ReqwestClient;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use crate::planner::Case;
use crate::executor::VUserMetrics;

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

/// State of the test controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControllerState {
    Running,
    Paused,
    Stopped,
}

/// HTTP Client wrapper
#[derive(Clone)]
struct HttpClient(ReqwestClient);

impl Client for HttpClient {
    type Param = &'static str;

    async fn execute<M: RequestMetrics>(&self, param: Self::Param) -> M {
        let start = Instant::now();
        let response = self.0.get(param).send().await;

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

trait Client {
    type Param;
    async fn execute<M: RequestMetrics>(&self, param: Self::Param) -> M;
}

/// Represents a single virtual user executing requests
struct VirtualUser<M: RequestMetrics, C: Client> {
    id: u8,
    client: C,
    metrics_sender: mpsc::Sender<M>,
    state_rx: watch::Receiver<ControllerState>,
}

impl<M: RequestMetrics, C: Client> VirtualUser<M, C> {
    /// Run the virtual user's request loop
    async fn run(mut self) {
        loop {
            // Check state
            let state = *self.state_rx.borrow();
            match state {
                ControllerState::Stopped => break,
                ControllerState::Paused => {
                    // Wait for state change
                    if self.state_rx.changed().await.is_err() {
                        break;
                    }
                    continue;
                }
                ControllerState::Running => {
                    // Execute request
                    //TODO: requests might be done in parallel not sequentially
                    let metrics = self.client.execute::<M>(&self.target_url).await;
                    let _ = self.metrics_sender.send(metrics).await;
                }
            }
        }
    }
}

/// Metrics collector that aggregates results
struct MetricsCollector<A: MetricsAggregator>(mpsc::Receiver<A::Request>);

impl<A: MetricsAggregator> MetricsCollector<A> {
    async fn collect(mut self) -> A {
        let mut aggregator = A::default();

        while let Some(metric) = self.0.recv().await {
            aggregator.add(metric);
        }

        aggregator.finalize();
        aggregator
    }
}

/// Controller that manages virtual users and runtime control
struct TestController<A: MetricsAggregator> {
    client: HttpClient,
    metrics_collector: MetricsCollector<A>,
    control_tx: mpsc::Sender<ControlCommand>,
    control_rx: mpsc::Receiver<ControlCommand>,
    state_tx: watch::Sender<ControllerState>,
    state_rx: watch::Receiver<ControllerState>,
    user_handles: Vec<JoinHandle<()>>,
    next_user_id: u8,
}
