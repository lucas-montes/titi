//! Virtual User - Abstract Action Execution Framework
//!
//! This module provides a generic VirtualUser that can execute any type of action,
//! not just HTTP requests. This allows for testing different protocols and systems:
//!
//! - HTTP/HTTPS requests (via `HttpAction`)
//! - gRPC calls (implement `Action` trait)
//! - WebSocket messages (implement `Action` trait)
//! - Database queries (implement `Action` trait)
//! - File operations (implement `Action` trait)
//! - Custom business logic (implement `Action` trait)
//!
//! # Architecture
//!
//! ```text
//! VirtualUser<A: Action>
//!  ├─ Action trait (what to execute)
//!  │   └─ Returns ActionMetrics
//!  ├─ RateLimiter (0.0 = unlimited)
//!  └─ Metrics channel (send to Executor)
//! ```
//!
//! # Example: HTTP Testing
//!
//! ```rust,ignore
//! use fuzzer::vuser::{create_http_vuser, HttpMetrics};
//! use fuzzer::client::create_http_client;
//! use fuzzer::planner::RequestTemplate;
//! use tokio::sync::mpsc;
//!
//! let client = create_http_client();
//! let template = RequestTemplate { /* ... */ };
//! let (metrics_tx, metrics_rx) = mpsc::channel::<HttpMetrics>(10_000);
//!
//! let mut vuser = create_http_vuser(
//!     1,              // id
//!     client,         // HTTP client
//!     template,       // what to request
//!     metrics_tx,     // where to send metrics
//!     100.0,          // 100 requests per second
//!     1,              // sequential execution
//! );
//!
//! // Run the VUser
//! tokio::spawn(async move {
//!     vuser.run().await;
//! });
//! ```
//!
//! # Example: Custom Action
//!
//! ```rust,ignore
//! use fuzzer::vuser::{Action, ActionMetrics, VirtualUser};
//! use std::pin::Pin;
//! use std::future::Future;
//!
//! #[derive(Clone)]
//! struct DatabaseAction {
//!     query: String,
//! }
//!
//! #[derive(Debug, Clone)]
//! struct DatabaseMetrics {
//!     vuser_id: u16,
//!     timestamp: Instant,
//!     duration: Duration,
//!     rows_affected: u64,
//!     success: bool,
//! }
//!
//! impl Action for DatabaseAction {
//!     type Metrics = DatabaseMetrics;
//!
//!     fn execute(&self, vuser_id: u16) -> Pin<Box<dyn Future<Output = Self::Metrics> + Send + 'static>> {
//!         let query = self.query.clone();
//!         Box::pin(async move {
//!             // Execute database query
//!             // Return metrics
//!         })
//!     }
//! }
//!
//! impl ActionMetrics for DatabaseMetrics {
//!     fn vuser_id(&self) -> u16 { self.vuser_id }
//!     fn timestamp(&self) -> Instant { self.timestamp }
//!     fn duration(&self) -> Duration { self.duration }
//!     fn is_success(&self) -> bool { self.success }
//! }
//! ```

use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::planner::RequestTemplate;
use crate::{client::RequestError, ratelimiter::RateLimiter};
use reqwest::{Client as ReqwestClient, StatusCode};


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



#[derive(Clone)]
pub struct HttpAction {
    client: ReqwestClient,
    template: RequestTemplate,
}

impl HttpAction {
    pub fn new(client: ReqwestClient, template: RequestTemplate) -> Self {
        Self { client, template }
    }
}

impl Action for HttpAction {
    type Metrics = HttpMetrics;

    async fn execute(&self, vuser_id: u16) -> Self::Metrics {
        let client = self.client.clone();
        let template = self.template.clone();

        let start = Instant::now();

        let request_builder = template.to_request_builder(&client);
        let result = request_builder.send().await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                HttpMetrics {
                    vuser_id,
                    timestamp: start,
                    duration: elapsed,
                    status_code: Some(response.status()),
                    error: None,
                }
            }
            Err(err) => HttpMetrics {
                vuser_id,
                timestamp: start,
                duration: elapsed,
                status_code: None,
                error: Some(RequestError::from(err)),
            },
        }
    }
}

/// Metrics from an HTTP request
#[derive(Debug, Clone)]
pub struct HttpMetrics {
    vuser_id: u16,
    timestamp: Instant,
    duration: std::time::Duration,
    status_code: Option<StatusCode>,
    error: Option<RequestError>,
}

impl ActionMetrics for HttpMetrics {
    fn vuser_id(&self) -> u16 {
        self.vuser_id
    }

    fn timestamp(&self) -> Instant {
        self.timestamp
    }

    fn duration(&self) -> std::time::Duration {
        self.duration
    }

    fn is_success(&self) -> bool {
        self.error.is_none() && self.status_code.is_some_and(|c| c.is_success())
    }
}

impl HttpMetrics {
    /// Get the HTTP status code (if available)
    pub fn status_code(&self) -> Option<u16> {
        self.status_code.map(|c| c.as_u16())
    }

    /// Get the categorized error (if any)
    pub fn error(&self) -> Option<RequestError> {
        self.error
    }
}


pub fn create_http_vuser(
    id: u16,
    client: ReqwestClient,
    template: RequestTemplate,
    metrics_tx: mpsc::Sender<HttpMetrics>,
    requests_per_second: usize,
    parallel_requests: u8,
) -> VirtualUser<HttpAction> {
    let action = HttpAction::new(client, template);
    VirtualUser::new(
        id,
        action,
        metrics_tx,
        requests_per_second,
        parallel_requests,
    )
}
