use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ScenarioId(String);

/// Main test scenario configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// Unique identifier for this scenario
    id: ScenarioId,
    /// Name of the test scenario
    name: String,
    /// Description of what this test does
    description: Option<String>,
    /// Target configuration
    target: TargetConfig,
    /// Test scenario type and settings
    scenario: Scenario,
}

/// Target system configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Base URL of the target system
    base_url: String,
    /// Optional authentication token
    auth_token: Option<String>,
    /// Request timeout in seconds
    timeout_seconds: u64,
    /// Custom headers to include in all requests
    headers: Option<Vec<(String, String)>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Scenario {
    LoadTesting(LoadTesting),
    Security(Security),
}

/// Security testing configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Security {
    /// List of attack types to test
    attack_types: Vec<AttackType>,
    /// Number of concurrent requests
    concurrent_requests: u32,
    /// Maximum number of total requests
    max_requests: Option<u64>,
    /// Endpoints to test (if empty, will discover from target)
    endpoints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackType {
    SqlInjection,
    XssInjection,
    CommandInjection,
    PathTraversal,
    AuthBypass,
    BrokenAccess,
    Csrf,
}

/// Load testing configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadTesting {
    /// Load pattern to use
    pattern: LoadPattern,
    /// Duration of the test in seconds
    duration_seconds: u64,
    /// Endpoints to test with their weights
    endpoints: Vec<EndpointWeight>,
    /// Think time between requests in milliseconds (per virtual user)
    think_time_ms: u64,
}

/// Load pattern configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoadPattern {
    /// Constant number of virtual users
    Constant {
        virtual_users: u32,
        requests_per_second: Option<u32>,
    },
    /// Ramp up from start to end users
    RampUp {
        start_users: u32,
        end_users: u32,
        ramp_duration_seconds: u64,
    },
    /// Spike pattern
    Spike {
        base_users: u32,
        spike_users: u32,
        spike_duration_seconds: u64,
    },
    /// Step pattern
    Step {
        start_users: u32,
        step_increment: u32,
        step_duration_seconds: u64,
        max_users: u32,
    },
}

/// Endpoint with weight for load distribution
#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointWeight {
    /// HTTP method
    method: String,
    /// Endpoint path
    path: String,
    /// Optional request body template
    body: Option<String>,
    /// Weight for this endpoint (higher = more requests)
    weight: u32,
}
