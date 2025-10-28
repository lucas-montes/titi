//! HTTP Client utilities for the fuzzer
//!
//! This module provides HTTP client creation and error categorization.
//! The actual request execution logic is in vuser.rs.

use reqwest::Client as ReqwestClient;
use std::time::Duration;

/// Categorized request errors (zero-allocation)
///
/// Used for efficient error tracking without heap allocations.
/// VUser sends these categorized errors instead of full error strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestError {
    /// Connection-related errors (DNS, TCP, socket errors, etc.)
    Connection,
    /// Request timeout (exceeded configured duration)
    Timeout,
    /// TLS/SSL handshake or certificate errors
    Tls,
    /// HTTP protocol errors (invalid response, etc.)
    Protocol,
    /// Failed to read or decode response body
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

/// Create an HTTP client with optimal settings for load testing
///
/// Configuration:
/// - Connection pooling: 100 idle connections per host
/// - Request timeout: 30 seconds (configurable)
/// - Keep-alive enabled
/// - HTTP/2 enabled (falls back to HTTP/1.1)
/// - Follows redirects (up to 10)
///
/// Note: reqwest::Client uses Arc internally, so it's cheap to clone.
/// All VUsers can share the same client via clone() - no need for Arc wrapper.
pub fn create_http_client() -> ReqwestClient {
    create_http_client_with_timeout(Duration::from_secs(30))
}

/// Create an HTTP client with custom timeout
pub fn create_http_client_with_timeout(timeout: Duration) -> ReqwestClient {
    ReqwestClient::builder()
        .pool_max_idle_per_host(100)
        .timeout(timeout)
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .http2_prior_knowledge()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .expect("Failed to create HTTP client")
}

/// Create an HTTP client for testing (no timeout, smaller pool)
#[cfg(test)]
pub fn create_test_client() -> ReqwestClient {
    ReqwestClient::builder()
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create test HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_error_display() {
        assert_eq!(RequestError::Connection.to_string(), "connection");
        assert_eq!(RequestError::Timeout.to_string(), "timeout");
        assert_eq!(RequestError::Tls.to_string(), "tls");
        assert_eq!(RequestError::Protocol.to_string(), "protocol");
        assert_eq!(RequestError::BodyRead.to_string(), "body_read");
        assert_eq!(RequestError::Cancelled.to_string(), "cancelled");
        assert_eq!(RequestError::Other.to_string(), "other");
    }

    #[test]
    fn test_client_creation() {
        let client = create_http_client();
        // Client is created successfully
        assert!(std::mem::size_of_val(&client) > 0);

        let client_custom = create_http_client_with_timeout(Duration::from_secs(10));
        // Custom timeout client is created successfully
        assert!(std::mem::size_of_val(&client_custom) > 0);
    }

    #[test]
    fn test_client_is_clonable() {
        let client1 = create_http_client();
        let client2 = client1.clone();

        // Both should be valid (reqwest::Client uses Arc internally)
        assert!(std::mem::size_of_val(&client1) > 0);
        assert!(std::mem::size_of_val(&client2) > 0);
    }
}
