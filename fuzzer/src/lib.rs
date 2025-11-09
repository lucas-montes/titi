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

use crate::configuration::Configuration;

mod configuration;



/// Planner that takes a Configuration and produces a series of Plans the planner is in charge of using the configuration, pass the information into an algorithm and generate a plan.
/// Each plan will represent a set of cases to be executed by the scheduler. A case can be a set of requests to be made, with specific parameters, headers, body, etc...
trait Planner: From<Configuration> + IntoIterator<Item = Self::Plan> {
    type Plan;
    fn plan(&mut self);
}

trait MetricsAggregator {
    type Metrics;
    type MetricsSnapshot;
    fn aggregate(&mut self, metrics: Self::Metrics);
    fn export(&self) -> Self::MetricsSnapshot;
}

/// Scheduler that takes Plans from the Planner and produces Cases to be executed. It collects metrics from one or more Executors and sends them to a MetricsAggregator.
trait Scheduler {
    type MetricsAggregator: MetricsAggregator;
    fn spawn(&mut self);
}

/// A Case to be executed by the Executor, a case should represent a generator of requests to be made.
trait Case {
    type Metrics;
    type Assertions;
}


/// Executor that takes care of running the cases, spawning virtual users, etc...
trait Executor {
    type Case: Case;
    type MetricsAggregator: MetricsAggregator;
}


/// Virtual user that will execute the requests
struct VUser;

Configuration
  ↓
Planner (generates Plans)
  ↓
Plan (complete blueprint: endpoints, ranges, load_profile, assertions, etc.)
  ↓
Scheduler (orchestrates execution)
  ├── Creates Executors (could be local or remote)
  ├── Distributes Plans across Executors
  ├── Collects metrics from all Executors
  └── Sends aggregated metrics to MetricsAggregator

  ↓ (per Executor)
Executor (runs locally or on remote machine)
  ├── Spawns VUsers based on Plan's LoadProfile
  ├── Manages VUser lifecycle (ramp-up, steady state, ramp-down)
  ├── Collects RequestMetrics from VUsers
  └── Sends metrics back to Scheduler

  ↓ (per VUser)
VUser (lightweight async task)
  ├── Generates requests from Plan
  ├── Executes HTTP/gRPC requests
  ├── Validates assertions
  └── Returns RequestMetrics

  ↓
MetricsAggregator (in-memory or persistent)
  └── Exports JSON snapshot


// Plan is the blueprint that contains ALL execution details
struct Plan {
    id: String,
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
    workflow: Workflow,  // linear request sequence
}

// Planner generates Plans from Configuration
trait Planner {
    fn plan(&self, config: Configuration) -> Vec<Plan>;
}

// Scheduler creates Executors and distributes Plans
trait Scheduler {
    type Executor: Executor;

    async fn schedule(
        &mut self,
        plans: Vec<Plan>,
        executor_factory: impl ExecutorFactory<Self::Executor>,
    ) -> SchedulerMetrics;
}

// ExecutorFactory allows creating local or remote executors
trait ExecutorFactory<E: Executor> {
    fn create_executor(&self) -> E;
}

// Executor spawns VUsers and runs a Plan
trait Executor {
    async fn execute(&self, plan: Plan) -> ExecutionMetrics;
}

// VUser executes individual requests
trait VUser {
    async fn execute_request(&self, request: Request, assertions: &[Assertion])
        -> RequestMetrics;
}

// Case represents a request generator
trait Case {
    fn generate_request(&self, index: usize) -> Request;
}

// MetricsAggregator collects all metrics
trait MetricsAggregator {
    fn aggregate(&mut self, metrics: ExecutionMetrics);
    fn export_json(&self) -> String;
}
