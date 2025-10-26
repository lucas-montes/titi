use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ScenarioId(String);

/// Main test scenario configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct TestScenario {
    /// Unique identifier for this scenario
    id: ScenarioId,

    /// Human-readable name
    name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// API specification
    spec: ApiSpec,

    /// Target API configuration
    target: TargetConfig,

    /// Execution settings
    execution: ExecutionConfig,

    /// Test phases to run
    phases: Vec<TestPhase>,

    /// Optional advanced settings
    #[serde(default)]
    advanced: AdvancedConfig,

    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

/// API Specification source
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ApiSpec {
    /// OpenAPI 3.x specification
    #[serde(rename = "openapi")]
    OpenApi {
        /// Version (e.g., "3.0.1", "3.1.0")
        version: String,
        /// YAML or JSON content
        content: String,
    },

    /// GraphQL schema
    #[serde(rename = "graphql")]
    GraphQL {
        /// GraphQL schema definition
        schema: String,
        /// Introspection query result (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        introspection: Option<String>,
    },

    /// Postman Collection v2.1
    #[serde(rename = "postman")]
    Postman {
        /// Collection JSON
        collection: String,
    },

    /// HAR (HTTP Archive) file
    #[serde(rename = "har")]
    Har {
        /// HAR JSON content
        content: String,
    },

    /// Manual endpoint definitions
    #[serde(rename = "manual")]
    Manual {
        /// List of manually defined endpoints
        endpoints: Vec<ManualEndpoint>,
    },
}

/// Manually defined endpoint
#[derive(Debug, Serialize, Deserialize)]
struct ManualEndpoint {
    path: String,
    method: HttpMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Vec<Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body_schema: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

#[derive(Debug, Serialize, Deserialize)]
struct Parameter {
    name: String,
    location: ParameterLocation,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

/// Target API configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Base URL (e.g., "https://api.example.com")
    pub base_url: String,

    /// Environment name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,

    /// Authentication configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,

    /// Custom headers to include in all requests
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Authentication configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    /// No authentication
    #[serde(rename = "none")]
    None,

    /// Basic authentication
    #[serde(rename = "basic")]
    Basic {
        username: String,
        password: String,
    },

    /// Bearer token
    #[serde(rename = "bearer")]
    Bearer {
        token: String,
    },

    /// API Key
    #[serde(rename = "api_key")]
    ApiKey {
        /// Where to put the key (header or query)
        location: ApiKeyLocation,
        /// Parameter name (e.g., "X-API-Key" or "api_key")
        name: String,
        /// The actual key value
        value: String,
    },

    /// OAuth 2.0
    #[serde(rename = "oauth2")]
    OAuth2 {
        /// Access token
        access_token: String,
        /// Optional refresh token
        #[serde(skip_serializing_if = "Option::is_none")]
        refresh_token: Option<String>,
        /// Token type (usually "Bearer")
        #[serde(default = "default_token_type")]
        token_type: String,
    },
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyLocation {
    Header,
    Query,
}

/// Execution configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum concurrent requests
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: usize,

    /// Rate limit (requests per second)
    #[serde(default = "default_requests_per_second")]
    pub requests_per_second: usize,

    /// Timeout per request (in seconds)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum total test duration (in seconds, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_seconds: Option<u64>,

    /// Retry on failure
    #[serde(default)]
    pub retry_on_failure: bool,

    /// Number of retries
    #[serde(default = "default_retry_count")]
    pub retry_count: usize,

    /// Delay between retries (milliseconds)
    #[serde(default = "default_retry_delay")]
    retry_delay_ms: u64,
}

fn default_concurrent_requests() -> usize { 10 }
fn default_requests_per_second() -> usize { 100 }
fn default_timeout() -> u64 { 30 }
fn default_retry_count() -> usize { 3 }
fn default_retry_delay() -> u64 { 1000 }

/// Test phases to execute
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TestPhase {
    /// Happy path testing with valid inputs
    #[serde(rename = "happy_path")]
    HappyPath {
        /// Number of test variations to generate
        #[serde(default = "default_test_count")]
        test_count: usize,
    },

    /// Security vulnerability testing
    #[serde(rename = "security")]
    Security {
        /// Specific attack types to test
        #[serde(default)]
        attacks: Vec<SecurityAttack>,

        /// Use fuzzing wordlists
        #[serde(default = "default_true")]
        use_wordlists: bool,
    },

    /// Compliance and standards testing
    #[serde(rename = "compliance")]
    Compliance {
        /// Standards to check
        #[serde(default)]
        standards: Vec<ComplianceStandard>,
    },

    /// Correctness and functional testing
    #[serde(rename = "correctness")]
    Correctness {
        /// Test CRUD operations
        #[serde(default = "default_true")]
        test_crud: bool,

        /// Test idempotency
        #[serde(default = "default_true")]
        test_idempotency: bool,

        /// Test business logic
        #[serde(default)]
        test_business_logic: bool,
    },

    /// Performance and load testing
    #[serde(rename = "performance")]
    Performance {
        /// Load test pattern
        pattern: LoadPattern,

        /// Virtual users or concurrent connections
        virtual_users: usize,

        /// Test duration in seconds
        duration_seconds: u64,
    },

    /// Boundary value testing
    #[serde(rename = "boundary")]
    Boundary {
        /// Test min/max values
        #[serde(default = "default_true")]
        test_limits: bool,

        /// Test edge cases
        #[serde(default = "default_true")]
        test_edge_cases: bool,
    },

    /// Custom fuzzing configuration
    #[serde(rename = "custom_fuzz")]
    CustomFuzz {
        /// Endpoints to fuzz (empty = all)
        #[serde(default)]
        endpoints: Vec<String>,

        /// Fuzzing techniques
        techniques: Vec<FuzzTechnique>,

        /// Maximum tests to generate
        #[serde(default = "default_fuzz_max")]
        max_tests: usize,
    },
}

fn default_test_count() -> usize { 10 }
fn default_true() -> bool { true }
fn default_fuzz_max() -> usize { 1000 }

/// Security attack types
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SecurityAttack {
    SqlInjection,
    NoSqlInjection,
    XssReflected,
    XssStored,
    CommandInjection,
    LdapInjection,
    XpathInjection,
    PathTraversal,
    Ssrf,
    Xxe,
    Csrf,
    MassAssignment,
    AuthBypass,
    PrivilegeEscalation,
    WeakCrypto,
    SensitiveDataExposure,
}

/// Compliance standards
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ComplianceStandard {
    /// OpenAPI specification compliance
    OpenApi,
    /// HTTP standards (RFC 7231, etc.)
    HttpStandards,
    /// REST constraints (HATEOAS, etc.)
    RestConstraints,
    /// CORS policy
    Cors,
    /// Security headers
    SecurityHeaders,
    /// Rate limiting
    RateLimiting,
    /// Pagination standards
    Pagination,
    /// Error format (RFC 7807)
    ErrorFormat,
    /// GDPR compliance
    Gdpr,
}

/// Load test patterns
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LoadPattern {
    /// Constant load
    Constant,
    /// Gradual ramp-up
    RampUp { from: usize, to: usize, duration_seconds: u64 },
    /// Sudden spike
    Spike { base: usize, spike: usize, spike_duration_seconds: u64 },
    /// Sustained load (soak test)
    Soak,
}

/// Fuzzing techniques
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FuzzTechnique {
    /// Random byte mutation
    RandomMutation,
    /// Type confusion (string as number, etc.)
    TypeConfusion,
    /// Boundary values
    BoundaryValues,
    /// Large payloads
    LargePayloads,
    /// Special characters
    SpecialCharacters,
    /// Null/empty values
    NullEmpty,
    /// Format strings
    FormatStrings,
    /// Unicode/encoding attacks
    UnicodeAttacks,
}

/// Advanced configuration options
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AdvancedConfig {
    /// Save full responses to disk
    #[serde(default)]
    save_responses: bool,

    /// Maximum response size to save (MB)
    #[serde(default = "default_max_response_size")]
    max_response_size_mb: usize,

    /// Follow HTTP redirects
    #[serde(default = "default_true")]
    follow_redirects: bool,

    /// Verify SSL certificates
    #[serde(default = "default_true")]
    verify_ssl: bool,

    /// HTTP proxy
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy: Option<String>,

    /// User agent string
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,

    /// Environment variables for dynamic values
    #[serde(default)]
    variables: HashMap<String, String>,

    /// Seed for random number generator (for reproducibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    random_seed: Option<u64>,
}

impl AdvancedConfig {
    pub fn verify_ssl(&self) -> bool {
        self.verify_ssl
    }

    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    pub fn proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }
}

fn default_max_response_size() -> usize { 10 }

/// Helper function to load configuration from JSON file
impl TestScenario {
    /// Get the scenario ID
    pub fn id(&self) -> &ScenarioId {
        &self.id
    }

    /// Get the scenario name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the API spec
    pub fn spec(&self) -> &ApiSpec {
        &self.spec
    }

    /// Get the target configuration
    pub fn target(&self) -> &TargetConfig {
        &self.target
    }

    /// Get the execution configuration
    pub fn execution(&self) -> &ExecutionConfig {
        &self.execution
    }

    /// Get the test phases
    pub fn phases(&self) -> &[TestPhase] {
        &self.phases
    }

    /// Get the advanced configuration
    pub fn advanced(&self) -> &AdvancedConfig {
        &self.advanced
    }

    /// Get the tags
    pub fn tags(&self) -> Option<&[String]> {
        self.tags.as_deref()
    }

    /// Load from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Load from JSON file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&content)?)
    }

    /// Save to JSON string (pretty-printed)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Save to JSON file
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal_config() {
        let json = r#"{
            "id": "test-001",
            "name": "Basic Security Test",
            "spec": {
                "type": "openapi",
                "data": {
                    "version": "3.0.1",
                    "content": "openapi: 3.0.0..."
                }
            },
            "target": {
                "base_url": "https://api.example.com"
            },
            "execution": {},
            "phases": [
                {
                    "type": "security",
                    "attacks": ["sql_injection", "xss_reflected"]
                }
            ]
        }"#;

        let scenario: TestScenario = serde_json::from_str(json).unwrap();
        assert_eq!(scenario.id, ScenarioId("test-001".into()));
        assert_eq!(scenario.target.base_url, "https://api.example.com");
    }

    #[test]
    fn test_deserialize_full_config() {
        let json = r#"{
            "id": "test-002",
            "name": "Comprehensive Test Suite",
            "description": "Full API testing with all phases",
            "spec": {
                "type": "openapi",
                "data": {
                    "version": "3.0.1",
                    "content": "openapi: 3.0.0..."
                }
            },
            "target": {
                "base_url": "https://api.example.com",
                "environment": "staging",
                "auth": {
                    "type": "bearer",
                    "token": "abc123"
                },
                "headers": {
                    "X-Custom-Header": "value"
                }
            },
            "execution": {
                "concurrent_requests": 20,
                "requests_per_second": 200,
                "timeout_seconds": 60,
                "retry_on_failure": true,
                "retry_count": 3
            },
            "phases": [
                {
                    "type": "happy_path",
                    "test_count": 50
                },
                {
                    "type": "security",
                    "attacks": ["sql_injection", "xss_reflected"],
                    "use_wordlists": true
                },
                {
                    "type": "performance",
                    "pattern": {
                        "ramp_up": {
                            "from": 10,
                            "to": 100,
                            "duration_seconds": 300
                        }
                    },
                    "virtual_users": 100,
                    "duration_seconds": 600
                }
            ],
            "advanced": {
                "save_responses": true,
                "verify_ssl": true,
                "variables": {
                    "USER_EMAIL": "test@example.com"
                }
            },
            "tags": ["api", "security", "performance"]
        }"#;

        let scenario: TestScenario = serde_json::from_str(json).unwrap();
        assert_eq!(scenario.phases.len(), 3);
        assert!(scenario.advanced.save_responses);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let scenario = TestScenario {
            id: ScenarioId("test-003".into()),
            name: "Test Scenario".to_string(),
            description: None,
            spec: ApiSpec::OpenApi {
                version: "3.0.1".to_string(),
                content: "openapi: 3.0.0...".to_string(),
            },
            target: TargetConfig {
                base_url: "https://api.example.com".to_string(),
                environment: None,
                auth: None,
                headers: HashMap::new(),
            },
            execution: ExecutionConfig {
                concurrent_requests: 10,
                requests_per_second: 100,
                timeout_seconds: 30,
                max_duration_seconds: None,
                retry_on_failure: false,
                retry_count: 3,
                retry_delay_ms: 1000,
            },
            phases: vec![
                TestPhase::HappyPath { test_count: 10 }
            ],
            advanced: AdvancedConfig::default(),
            tags: None,
        };

        let json = scenario.to_json().unwrap();
        let deserialized: TestScenario = serde_json::from_str(&json).unwrap();

        assert_eq!(scenario.id, deserialized.id);
        assert_eq!(scenario.name, deserialized.name);
    }
}
