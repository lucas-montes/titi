use serde::Serialize;

use crate::config::Configuration;
use crate::vuser::Action;
use std::collections::HashMap;
use std::time::Duration;

/// Test planner trait - generates test scenarios lazily using various algorithms
/// (IPOG, IPOG-C, T-wise Adaptive, Boundary Analysis, etc.)
///
/// A planner generates high-level test scenarios, each of which can generate
/// multiple concrete test cases through parameter space exploration.
pub trait TestPlanner: Iterator<Item = TestScenario<Self::Action>> + From<Configuration>{
    type Action: Action;

    /// Get algorithm name
    fn algorithm_name(&self) -> &'static str;

    /// Provide feedback to adaptive planners (for dynamic test generation)
    fn update(&mut self, _feedback: PlannerFeedback) {}
}

/// Feedback from executor to planner for adaptive test generation
#[derive(Debug, Serialize)]
pub struct PlannerFeedback {
    scenario_id: String,
    success_rate: f64,
    error_types: HashMap<ErrorCategory, usize>,
    recommendation: FeedbackRecommendation,
}

/// Recommendation for planner adaptation
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum FeedbackRecommendation {
    /// Low failure rate - try more complex test cases
    IncreaseComplexity,
    /// High failure rate - simplify test cases
    DecreaseComplexity,
    /// Focus on specific error area
    FocusOnErrorArea,
    /// Continue with current strategy
    Continue,
}

/// Error categories (protocol-agnostic)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub enum ErrorCategory {
    Timeout,
    Connection,
    Authentication,
    RateLimited,
    ServerError,
    ClientError,
    Unknown,
}

/// A test scenario represents a high-level test strategy targeting one or more endpoints.
/// Each scenario defines a parameter space and generates multiple concrete test cases lazily.
#[derive(Debug, Clone)]
pub struct TestScenario<A: Action> {
    pub scenario_id: String,
    pub description: String,

    /// Parameter space to explore (e.g., combinations of headers, payloads, query params)
    pub parameter_space: ParameterSpace,

    /// Constraints on parameter combinations (e.g., "if auth=none, then user_id must be empty")
    pub constraints: Vec<Constraint>,

    /// Factory function that creates an Action from a concrete parameter combination
    pub action_factory: fn(&ParameterCombination) -> A,

    /// Stopping conditions for this scenario
    pub stopping_conditions: StoppingConditions,
}

/// Parameter space definition - abstract representation of test dimensions
#[derive(Debug, Clone)]
pub struct ParameterSpace {
    /// Parameter dimensions (e.g., "method", "auth_type", "payload_size")
    pub dimensions: HashMap<String, Vec<ParameterValue>>,
}

/// A parameter value (protocol-agnostic)
#[derive(Debug, Clone)]
pub enum ParameterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

/// A concrete combination of parameter values
pub type ParameterCombination = HashMap<String, ParameterValue>;

/// Constraint on parameter combinations (e.g., "if A then B", "not (C and D)")
#[derive(Debug, Clone)]
pub struct Constraint {
    pub description: String,
    pub validator: fn(&ParameterCombination) -> bool,
}

/// Stopping conditions for a scenario
#[derive(Debug, Clone, Copy)]
pub struct StoppingConditions {
    /// Maximum number of cases to generate (None = explore entire parameter space)
    pub max_cases: Option<usize>,

    /// Stop after N consecutive successes
    pub stop_after_successes: Option<usize>,

    /// Stop after N failures
    pub stop_after_failures: Option<usize>,

    /// Maximum duration for this scenario
    pub max_duration: Option<Duration>,
}

impl<A: Action> TestScenario<A> {
    /// Create an iterator that lazily generates test cases from this scenario
    pub fn into_cases(self) -> CaseIterator<A> {
        CaseIterator::new(self)
    }
}

/// Iterator that lazily generates test cases from a scenario's parameter space
pub struct CaseIterator<A: Action> {
    scenario: TestScenario<A>,
    current_index: usize,
    total_cases: Option<usize>,
    // Internal state for iterating through parameter combinations
    // (actual implementation depends on algorithm: IPOG, pairwise, etc.)
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Action> CaseIterator<A> {
    fn new(scenario: TestScenario<A>) -> Self {
        // Implementation details omitted
        Self {
            scenario,
            current_index: 0,
            total_cases: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<A: Action> Iterator for CaseIterator<A> {
    type Item = TestCase<A>;

    fn next(&mut self) -> Option<Self::Item> {
        // Implementation details omitted
        todo!("CaseIterator::next")
    }
}

/// A concrete test case with all parameters materialized
#[derive(Debug)]
pub struct TestCase<A: Action> {
    pub case_id: String,
    pub scenario_id: String,
    pub action: A,
    pub parameters: ParameterCombination,
    pub assertions: Vec<AssertionRule>,
}

/// Assertion to validate action result (protocol-agnostic)
#[derive(Debug, Clone)]
pub struct AssertionRule {
    pub description: String,
    // Validator will be implemented later - takes ActionMetrics and returns bool
}

// ============================================================================
// OLD ARCHITECTURE - To be phased out
// ============================================================================

/// A single test case generated by the planner
#[derive(Debug, Clone)]
pub struct Case {
    pub id: String,
    pub name: String,
    pub request_template: RequestTemplate,
    pub executor_type: ExecutorType,
    pub assertions: Vec<Assertion>,
    pub stopping_conditions: Vec<StoppingCondition>,
}

/// Request template for execution
#[derive(Debug, Clone)]
pub struct RequestTemplate {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub query_params: Vec<(String, String)>,
}

impl RequestTemplate {
    /// Convert template to reqwest RequestBuilder
    /// This avoids doing expensive request building work repeatedly
    pub fn to_request_builder(&self, client: &reqwest::Client) -> reqwest::RequestBuilder {
        // Start with method
        let mut builder = match self.method {
            Method::Get => client.get(&self.url),
            Method::Post => client.post(&self.url),
            Method::Put => client.put(&self.url),
            Method::Delete => client.delete(&self.url),
            Method::Patch => client.patch(&self.url),
            Method::Head => client.head(&self.url),
            Method::Options => client.request(reqwest::Method::OPTIONS, &self.url),
        };

        // Add headers
        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }

        // Add query params
        if !self.query_params.is_empty() {
            builder = builder.query(&self.query_params);
        }

        // Add body if present
        if let Some(body) = &self.body {
            builder = builder.body(body.clone());
        }

        builder
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// Execution strategy for a case
#[derive(Debug, Clone)]
pub enum ExecutorType {
    /// Execute once
    SingleShot,
    /// Constant number of VUsers
    ConstantVus { vus: usize, duration: Duration },
    /// Ramping VUsers over stages
    RampingVus { stages: Vec<Stage> },
    /// Constant arrival rate (open model)
    ConstantArrivalRate { rate: f64, duration: Duration, max_vus: usize },
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub target_vus: usize,
    pub duration: Duration,
}

/// Assertions to validate during/after execution
#[derive(Debug, Clone)]
pub enum Assertion {
    StatusCode(u16),
    StatusRange(u16, u16),
    BodyContains(String),
    HeaderExists(String),
    P50ResponseTime { threshold: Duration },
    P95ResponseTime { threshold: Duration },
    P99ResponseTime { threshold: Duration },
    SuccessRate { min: f64 },
    ErrorRate { max: f64 },
}

/// Conditions that trigger early stopping of a case
#[derive(Debug, Clone)]
pub enum StoppingCondition {
    FailureRate { threshold: f64 },
    ConsecutiveErrors { count: usize },
    P95ResponseTime { threshold: Duration },
    P99ResponseTime { threshold: Duration },
    TotalErrors { max: usize },
    NoSuccessFor { duration: Duration },
}

// Example planner implementation: Boundary Value Analysis
pub struct BoundaryPlanner {
    config: Configuration,
    cases: Vec<Case>,
    current: usize,
}

// TODO: Update BoundaryPlanner to implement new TestPlanner trait
// impl TestPlanner for BoundaryPlanner {
//     type Action = HttpAction; // or other Action type
//
//     fn from_config(config: Configuration) -> Self { ... }
//     fn algorithm_name(&self) -> &'static str { ... }
// }
//
// impl Iterator for BoundaryPlanner {
//     type Item = TestScenario<Self::Action>;
//     fn next(&mut self) -> Option<Self::Item> { ... }
// }
