// We have 5 layers for the fuzzer
// the first layer is the configuation that we receive from the user. This can be in asy shape, logs, json, yaml, manually entered, etc...
// We still need to add more information to the configuration but he idea is to have a high level representation of what the user wants to do
// The second layer is the planner that takes the configuration and creates a plan of what to do, we'll use some algorithms like:
// IPOG-C: Enhances IPOG with constraint handling for complex systems, improving efficiency in t-way testing.
// AI-Driven Testing (e.g., EvoSuite, DeepCT): Uses machine learning and genetic algorithms to optimize test case generation, adapting to code complexity.
// ACTS+: Advanced version of NIST’s ACTS, integrating constraint solvers and parallel processing for faster t-way testing.
// T-wise Adaptive Testing: Dynamically adjusts t-way coverage based on system behavior, reducing test suite size.
// And simple assertions, boundries checks, edge cases, etc...
// The planner will create cases with the things that we want to test and validate
// The third layer is the scheduler that will take the cases from the planner and decide when to run them, in parallel or in sequence, etc...
// The scheduler is also responsible to send the metrics to the metrics stream which is in talks with the website (and maybe an API)
// The fourth layer is the executor that will take care of running the cases, spawning virtual users, rate limiting, etc...
// The fifth layer is the virtual users that will execute the requests and send the metrics back to the executor


mod ratelimiter;
use std::sync::mpsc;

mod configuration {

    use std::path::PathBuf;

    /// Configuration received from the user specifiying endpoint, schema, auth, etc...
    pub struct Configuration {
        schema: Schema,
        tests: Vec<Test>, // TODO: add more fields
    }

    enum Schema{
        OpenAPI(PathBuf),
        GraphQL(PathBuf),
        Custom(PathBuf),
    }

    struct Test {
        endpoint: String,
        objectives: Vec<Objective>,
    }

    enum Objective {
        Security(Security),
        Reliability(Reliability),
        Performance(Performance),
    }
    struct Security {
        // Security specific fields
    }
    struct Reliability {
        // Reliability specific fields
    }
    struct Performance {
        // Performance specific fields
    }

}

mod algorithms {
    trait Algorithm {}
}

mod planner {

    /// Planner that takes a Configuration and produces a series of Plans the planner is in charge of using the configuration, pass the information into an algorithm and generate a plan.
    /// Each plan will represent a set of cases to be executed by the scheduler. A case can be a set of requests to be made, with specific parameters, headers, body, etc...
    trait Planner: From<crate::configuration::Configuration> + IntoIterator<Item = Self::Plan> {
        type Plan;
        fn plan(&mut self);
    }

    struct PlanId(String);

    // Plan is the blueprint that contains ALL execution details
    pub struct Plan {
        id: PlanId,
        seed: u64,
        endpoint: Endpoint,
        test_purpose: TestPurpose,
        algorithm_config: AlgorithmConfig,

        // Parameter generation
        parameter_ranges: ParameterSpace,
        constraints: ConstraintSet,

        // Execution profile
        load_profile: LoadProfile,

        // Validation
        assertions: Vec<Assertion>,

        // Workflow
        workflow: Workflow,
    }

    struct Workflow;
    struct Assertion;
    struct LoadProfile;
    struct ConstraintSet;
    struct ParameterSpace;
    struct AlgorithmConfig;
    struct TestPurpose;
    struct Endpoint;
}

mod metrics {
    pub trait MetricsAggregator {
        type Metrics;
        type MetricsSnapshot;
        fn aggregate(&mut self, metrics: Self::Metrics);
        fn export(&self) -> Self::MetricsSnapshot;
    }
}

mod scheduler {
    /// Scheduler that takes Plans from the Planner and produces Cases to be executed. It collects metrics from one or more Executors and sends them to a MetricsAggregator.

    struct Scheduler{
        rx
    }
}

mod executor {
    /// Executor that takes care of running the cases, spawning virtual users, etc...
    struct Executor {
        plan: crate::planner::Plan,
    }
}

mod vuser {
    pub struct VirtualUser<A: Action> {
        id: u16,
        action: A,
        metrics_tx: std::sync::mpsc::Sender<A::Metrics>,
        rate_limiter: crate::ratelimiter::RateLimiter,
        /// Number of parallel actions this VUser should execute (1 = sequential)
        parallel_actions: u8,
    }
}
