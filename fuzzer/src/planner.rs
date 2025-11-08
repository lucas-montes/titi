use crate::algorithms::Algorithm;
use crate::config::Configuration;
use crate::vuser::Action;




/// Test planner trait - generates test scenarios lazily using various algorithms
/// (IPOG, IPOG-C, T-wise Adaptive, Boundary Analysis, etc.)
///
/// A planner generates high-level test scenarios, each of which can generate
/// multiple concrete test cases through parameter space exploration.
pub trait Planner: Iterator<Item = TestScenario<Self::Action>> + From<Configuration> {
    type Action: Action;
    type Algorithm: Algorithm;

    /// Provide feedback to adaptive planners (for dynamic test generation)
    fn update(&mut self, _feedback: PlannerFeedback) {}
}


/// Feedback from executor to planner for adaptive test generation
#[derive(Debug)]
pub struct PlannerFeedback;


/// A test scenario represents a high-level test strategy targeting one or more endpoints.
/// Each scenario defines a parameter space and generates multiple concrete test cases lazily.
#[derive(Debug, Clone)]
pub struct TestScenario<A: Action> {
    scenario_id: String,
    description: String,

}


impl<A: Action> TestScenario<A> {
    /// Create an iterator that lazily generates test cases from this scenario
    fn into_cases(self) -> CaseIterator<A> {
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
    case_id: String,
    scenario_id: String,
    action: A,
    parameters: ParameterCombination,
    assertions: Vec<AssertionRule>,
}

/// Assertion to validate action result (protocol-agnostic)
#[derive(Debug, Clone)]
struct AssertionRule {
    description: String,
    // Validator will be implemented later - takes ActionMetrics and returns bool
}

/// Request template for execution
#[derive(Debug, Clone)]
pub struct RequestTemplate {
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
     body: Option<Vec<u8>>,
    query_params: Vec<(String, String)>,
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
