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

mod http;
mod ratelimiter;

pub mod configuration {

    use std::path::PathBuf;

    /// Configuration received from the user specifiying endpoint, schema, auth, etc...
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Configuration {
        schema: Schema,
        tests: Vec<Test>, // TODO: add more fields
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    enum Schema {
        OpenAPI(PathBuf),
        GraphQL(PathBuf),
        Custom(PathBuf),
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    enum Test {
        Security(Security),
        Reliability(Reliability),
        Performance(Performance),
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    struct Security {
        // Security specific fields
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    struct Reliability {
        // Reliability specific fields
    }
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
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
}

mod metrics {
    pub trait MetricsAggregator: Default {
        type Metrics;
        type MetricsSnapshot;
        fn aggregate(&mut self, metrics: Self::Metrics);
        fn export(&self) -> Self::MetricsSnapshot;
    }
}

mod scheduler {
    /// Scheduler that takes Plans from the Planner and produces Cases to be executed. It collects metrics from one or more Executors and sends them to a MetricsAggregator.

    struct Scheduler {}
}

/// Executor Module
///
/// This module orchestrates the execution of Virtual Users and manages metrics aggregation.
/// An Executor is responsible for:
///
/// 1. **VUser Lifecycle Management**: Spawning VUsers according to execution stages (ramp-up, steady state, ramp-down)
/// 2. **Real-time Metrics Collection**: Receiving metrics from all VUsers and aggregating them
/// 3. **Granular Control**: Sending stop signals to individual VUsers based on decision logic
/// 4. **Error Handling**: Detecting and logging panics in VUser tasks
/// 5. **Scheduler Communication**: Reporting aggregated metrics back to Scheduler
///
/// # Architecture Overview
///
/// - **Executor<M>**: Generic over a `MetricsAggregator` type (`M`). Holds execution state and metrics.
/// - **ExecutorId**: Unique identifier for this executor (for distributed scenarios).
/// - **ExecutorConfig**: Configuration for how to execute (stages, rate limits, max errors).
/// - **ExecutorSignal**: Signals received from Scheduler (e.g., Stop).
///
/// # Type Safety & Monomorphization
///
/// `Executor<M>` is generic over the `MetricsAggregator` trait to enable:
/// - Different metrics aggregation strategies at compile-time
/// - Type alignment: metrics from `VUser<G>` flow to `Executor<M>` where `M::Metrics` matches
/// - No runtime dispatch or trait objects in executor core
///
/// # Concurrency Model
///
/// The Executor runs a main async loop using `tokio::select!` that:
/// - Collects metrics from VUsers (real-time)
/// - Monitors VUser task completion (handles panics)
/// - Respects execution stage duration
/// - Can stop individual VUsers based on metrics analysis
///
/// Uses `JoinSet` for efficient concurrent task management:
/// - More efficient than `Vec<JoinHandle>` for many tasks
/// - Integrates naturally with `tokio::select!`
/// - Built-in panic detection
///
/// # Communication Channels
///
/// - **metrics_rx/metrics_tx**: VUsers → Executor (action metrics)
/// - **vusers_signal_txs**: Executor → VUsers (control signals)
/// - **scheduler_metrics_tx**: Executor → Scheduler (aggregated metrics)
/// - **scheduler_signals_rx**: Scheduler → Executor (stop, etc.)
///
/// # Example Flow
///
/// ```ignore
/// let executor = Executor::new(
///     executor_id,
///     config,
///     scheduler_metrics_tx,
///     scheduler_signals_rx,
/// );
///
/// // Run stages, spawning VUsers and collecting metrics
/// executor.run().await;
///
/// // Metrics are aggregated in-memory and sent to Scheduler
/// ```
mod executor {
    use std::sync::atomic::AtomicU16;

    use tokio::task::JoinSet;

    use crate::{metrics::MetricsAggregator, vuser::VUserSignal};
    use std::fmt;

    /// Unique identifier for an Executor
    ///
    /// Used for:
    /// - Logging and debugging
    /// - Distributed scenarios (identifying which machine/process is executing)
    /// - Correlating metrics with their source executor
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct ExecutorId(u16);

    impl fmt::Display for ExecutorId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "VUser#{}", self.0)
        }
    }

    impl From<u16> for ExecutorId {
        fn from(id: u16) -> Self {
            ExecutorId(id)
        }
    }

    /// Configuration for how an Executor runs VUsers
    ///
    /// Specifies the execution profile, stop conditions, and validation criteria.
    ///
    /// # Fields
    ///
    /// - `stages`: Load profile stages (ramp-up, steady state, ramp-down, etc.)
    ///   Each stage defines target VU count and duration.
    /// - `max_errors`: Maximum error count before stopping execution.
    ///   `None` means unlimited errors (only stop by duration).
    /// - `assertions`: Validation assertions to check on action metrics.
    ///   e.g., "HTTP 200", "response time < 500ms"
    /// - `rate_limit`: Global rate limit (requests/sec across all VUsers).
    ///   `None` means no global limit (VUsers rate-limit independently).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ExecutorConfig {
    ///     stages: vec![
    ///         Stage { duration: 10s, target_vus: 10 },  // Ramp up
    ///         Stage { duration: 30s, target_vus: 50 },  // Steady state
    ///         Stage { duration: 10s, target_vus: 0 },   // Ramp down
    ///     ],
    ///     max_errors: Some(100),  // Stop if 100+ errors
    ///     assertions: vec![
    ///         Assertion::HttpStatus(200),
    ///         Assertion::ResponseTime(Duration::from_millis(500)),
    ///     ],
    ///     rate_limit: None,  // VUsers rate-limit independently
    /// };
    /// ```
    pub struct ExecutorConfig {
        stages: Vec<Stage>,
        max_errors: Option<usize>,
        assertions: Vec<Assertion>,
        rate_limit: Option<f64>,
    }
    //TODO/ maybe add a From

    /// A stage in the execution profile
    ///
    /// Defines target VU count and duration for a phase of execution.
    /// Typically used for ramp-up, steady state, and ramp-down phases.
    struct Stage;

    /// An assertion/validation rule for action metrics
    ///
    /// Assertions are checked on each action's metrics to validate correct behavior.
    /// e.g., HTTP status code, response time, specific response content
    struct Assertion;

    /// Signals sent from Scheduler to Executor for control flow
    ///
    /// # Variants
    ///
    /// - `Stop`: Immediately stop execution. Executor will:
    ///   - Send stop signals to all running VUsers
    ///   - Wait for VUsers to complete gracefully
    ///   - Report final metrics to Scheduler
    ///
    /// # Future Extensions
    ///
    /// - `Pause`: Temporarily stop new VUser spawning (resume later)
    /// - `ReportMetrics`: Force immediate metrics snapshot
    /// - `AdjustRate`: Change global rate limit dynamically
    #[derive(Debug, Clone, Copy)]
    enum ExecutorSignal {
        Stop,
        // Future: Pause, ReportMetrics, AdjustRate(f64), etc.
    }

    /// Executor - orchestrates VUser lifecycle and metrics collection
    ///
    /// # Generic Parameter
    ///
    /// - `M: MetricsAggregator` - The metrics aggregation strategy. Different implementations
    ///   can aggregate metrics differently (summary stats, histograms, raw data, etc.)
    ///   Type ensures `M::Metrics` matches `VUser<G>::ActionMetrics`.
    ///
    /// # Responsibilities
    ///
    /// 1. **Spawn VUsers**: Create VirtualUser instances for each stage, with partitioned generators
    /// 2. **Manage Lifecycle**: Track spawned VUser tasks via JoinSet
    /// 3. **Collect Metrics**: Receive metrics from VUsers in real-time via metrics_rx
    /// 4. **Aggregate**: Accumulate metrics into `self.metrics` aggregator
    /// 5. **Monitor**: Detect VUser panics via JoinSet
    /// 6. **Control**: Send stop signals to VUsers via vusers_signal_txs
    /// 7. **Report**: Send aggregated metrics to Scheduler via scheduler_metrics_tx
    ///
    /// # Field Breakdown
    ///
    /// - `id`: Unique executor identifier (for logging and distributed scenarios)
    /// - `user_counter`: Atomic counter for generating unique VUser IDs
    /// - `metrics`: In-memory metrics aggregator (holds accumulated state)
    /// - `config`: Execution configuration (stages, max errors, assertions, rate limit)
    /// - `vusers_tasks`: JoinSet of running VUser tasks
    /// - `vusers_signal_txs`: Signal senders to all VUsers (for stop, etc.)
    /// - `metrics_rx`: Receiver for metrics from all VUsers
    /// - `metrics_tx`: Sender for metrics (VUsers send here)
    /// - `scheduler_metrics_tx`: Sender to Scheduler (reports final/periodic metrics)
    /// - `scheduler_signals_rx`: Receiver for signals from Scheduler (e.g., Stop)
    ///
    /// # Concurrency Model
    ///
    /// The Executor runs a main async loop using `tokio::select!` that listens for:
    /// 1. **Metrics arrivals**: Real-time metrics from VUsers → aggregate and analyze
    /// 2. **VUser completions**: JoinSet yields completed VUsers → detect panics
    /// 3. **Stage duration**: Timer expires → move to next stage or stop
    /// 4. **Scheduler signals**: Scheduler sends stop signal → shutdown gracefully
    ///
    /// This allows concurrent metric collection, task management, and signal handling.
    ///
    /// # Metrics Flow
    ///
    /// ```ignore
    /// VUser -> metrics_tx -> metrics_rx -> Executor.metrics.aggregate() -> scheduler_metrics_tx -> Scheduler
    /// ```
    ///
    /// Metrics flow one-way (no feedback from aggregator back to VUser).
    ///
    /// # Scaling Characteristics
    ///
    /// - **VUser count**: Limited by OS/tokio async task limits (typically millions)
    /// - **Metrics throughput**: All VUsers send to single `metrics_rx` channel
    ///   - Could bottleneck at very high throughput
    ///   - Future: could shard metrics channels or use lock-free aggregation
    /// - **Signal distribution**: All VUsers share single `vusers_signal_txs` Vec
    ///   - O(1) to send signal to specific VUser
    ///   - O(N) to stop all VUsers
    pub struct Executor<M: MetricsAggregator> {
        id: ExecutorId,
        user_counter: AtomicU16,
        metrics: M,
        config: ExecutorConfig,
        vusers_tasks: JoinSet<()>,
        vusers_signal_txs: Vec<tokio::sync::mpsc::Sender<VUserSignal>>,
        metrics_rx: tokio::sync::mpsc::Receiver<M::Metrics>,
        metrics_tx: tokio::sync::mpsc::Sender<M::Metrics>,
        scheduler_metrics_tx: tokio::sync::mpsc::Sender<M::Metrics>,
        scheduler_signals_rx: tokio::sync::mpsc::Receiver<ExecutorSignal>,
    }

    impl<M: MetricsAggregator> Executor<M> {
        /// Create a new Executor
        ///
        /// # Arguments
        ///
        /// - `id`: Unique identifier for this executor
        /// - `config`: Execution configuration (stages, max errors, assertions)
        /// - `scheduler_metrics_tx`: Channel sender for reporting metrics to Scheduler
        /// - `scheduler_signals_rx`: Channel receiver for control signals from Scheduler
        ///
        /// Creates internal metrics channel with capacity of 100 (can buffer 100 metric messages).
        ///
        /// # Type Constraints
        ///
        /// - `M: MetricsAggregator` - Must implement Default (initialized as `M::default()`)
        /// - `M::Metrics` must match what VUsers send (enforced at compile-time)
        ///
        /// # Example
        ///
        /// ```ignore
        /// let (scheduler_metrics_tx, scheduler_metrics_rx) = tokio::sync::mpsc::channel(1000);
        /// let (scheduler_signals_tx, scheduler_signals_rx) = tokio::sync::mpsc::channel(1);
        ///
        /// let executor = Executor::new(
        ///     0,                              // Executor ID
        ///     config,                         // ExecutorConfig
        ///     scheduler_metrics_tx,
        ///     scheduler_signals_rx,
        /// );
        ///
        /// executor.run().await;
        /// ```
        pub fn new(
            id: impl Into<ExecutorId>,
            config: impl Into<ExecutorConfig>,
            scheduler_metrics_tx: tokio::sync::mpsc::Sender<M::Metrics>,
            scheduler_signals_rx: tokio::sync::mpsc::Receiver<ExecutorSignal>,
        ) -> Self {
            let (metrics_tx, metrics_rx) = tokio::sync::mpsc::channel(100);
            Executor {
                id: id.into(),
                user_counter: AtomicU16::default(),
                metrics: M::default(),
                vusers_tasks: JoinSet::new(),
                vusers_signal_txs: Vec::new(),
                metrics_rx,
                metrics_tx,
                scheduler_metrics_tx,
                scheduler_signals_rx,
                config: config.into(),
            }
        }

        /// Run the Executor - main execution loop
        ///
        /// Orchestrates the complete execution lifecycle:
        ///
        /// 1. **Iterate through stages** in `self.config.stages`
        /// 2. **Spawn VUsers** for each stage (progressively ramping up/down)
        /// 3. **Collect metrics** in real-time from all running VUsers
        /// 4. **Aggregate metrics** using `self.metrics.aggregate()`
        /// 5. **Analyze** metrics to decide if should stop (e.g., max errors exceeded)
        /// 6. **Monitor VUsers**: Detect panics via JoinSet completion
        /// 7. **Respect timeouts**: Stop after stage duration or on Scheduler signal
        /// 8. **Cleanup**: Stop remaining VUsers, await all task completion
        ///
        /// # Concurrency
        ///
        /// Uses `tokio::select!` to handle three concurrent events:
        /// - Metrics arrival from VUsers
        /// - VUser task completion (panic detection)
        /// - Stage duration timeout or Scheduler signal
        ///
        /// Ensures efficient async handling without blocking.
        ///
        /// # Control Flow
        ///
        /// ```ignore
        /// for each stage in config.stages {
        ///     spawn VUsers to reach stage.target_vus
        ///     loop {
        ///         select! {
        ///             metrics => { aggregate and analyze },
        ///             task_completed => { handle panic if any },
        ///             timeout => { move to next stage },
        ///             scheduler_signal => { handle stop/pause },
        ///         }
        ///     }
        /// }
        /// ```
        ///
        /// # Error Handling
        ///
        /// - VUser panics are logged but don't abort execution
        /// - Metrics channel closure is handled gracefully
        /// - Scheduler signal is respected (immediate shutdown)
        ///
        /// # Panics
        ///
        /// Can panic if:
        /// - RateLimiter creation fails
        /// - Channel operations fail unexpectedly (shouldn't happen in normal flow)
        pub async fn run(&mut self) {
            // ... spawn VUsers with self.vuser_tasks.spawn(vuser.run())

            // Collect metrics while awaiting completions
            loop {
                tokio::select! {
                    Some(metrics) = self.metrics_rx.recv() => {
                        self.metrics.aggregate(metrics);
                        if self.should_stop() {
                            self.stop_all_vusers().await;
                        }
                    }
                    result = self.vusers_tasks.join_next(), if !self.vusers_tasks.is_empty() => {
                        if let Some(Err(e)) = result {
                            eprintln!("VUser panicked: {:?}", e);
                        }
                    }
                    _ = tokio::time::sleep(self.stage_duration) => {
                        self.stop_all_vusers().await;
                        break;
                    }
                }
            }

            // All tasks finished
        }
    }
}

/// Virtual User Module
///
/// This module defines the core abstractions for executing actions (requests) and collecting metrics.
/// Virtual users are lightweight async tasks that independently generate and execute actions against
/// a target system, simulating concurrent user behavior for load testing, security testing, and
/// API exploration.
///
/// # Architecture Overview
///
/// - **VirtualUser<G>**: The main executor, generic over an `ActionGenerator` type. Each VUser
///   owns its own generator (partition of test space), rate limiter, and signal receiver.
///
/// - **ActionGenerator**: A trait extending `Iterator` that lazily produces `Action`s. Each VUser
///   has a partitioned generator covering a portion of the test space.
///
/// - **Action**: A trait representing an executable operation (HTTP request, gRPC call, etc.) that
///   produces associated metrics when executed.
///
/// - **VUserSignal**: Signals sent from the Executor to control VUser behavior (e.g., Stop).
///
/// # Type Safety & Monomorphization
///
/// The module uses generic parameters (`G: ActionGenerator`) throughout to ensure:
/// - **Compile-time type safety**: No runtime type erasure or `dyn` trait objects
/// - **Monomorphic code generation**: Each concrete generator type produces specialized code
/// - **Zero-cost abstractions**: Full optimization by LLVM
///
/// # Concurrency Model
///
/// VUsers operate independently as spawned async tasks:
/// - Each VUser receives actions from its partitioned generator
/// - Rate limiting is applied per-VUser (distributed, not centralized)
/// - Metrics are sent immediately after each action execution
/// - Stop signals are received via a channel, checked in the main `tokio::select!` loop
///
/// # Example Flow
///
/// ```ignore
/// // Executor creates VUser
/// let (signal_tx, signal_rx) = tokio::sync::mpsc::channel(1);
/// let (metrics_tx, metrics_rx) = tokio::sync::mpsc::channel(100);
/// let vuser = VirtualUser::new(
///     id,
///     my_generator,
///     metrics_tx,
///     signal_rx,
///     requests_per_second,
/// );
///
/// // VUser runs in spawned task
/// tokio::spawn(vuser.run());
///
/// // Executor can send stop signal
/// signal_tx.send(VUserSignal::Stop).ok();
/// ```
mod vuser {
    use std::{fmt, time::Duration};

    /// Unique identifier for a VirtualUser
    ///
    /// A newtype wrapper around `u16` for type safety and semantic clarity.
    /// Implements `Display` for logging and debugging.
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct VirtualUserId(u16);

    impl fmt::Display for VirtualUserId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "VUser#{}", self.0)
        }
    }

    impl From<u16> for VirtualUserId {
        fn from(id: u16) -> Self {
            VirtualUserId(id)
        }
    }

    /// Action trait - represents a single executable operation
    ///
    /// An `Action` is any operation that a VirtualUser can perform, typically:
    /// - HTTP requests (GET, POST, PUT, DELETE, etc.)
    /// - gRPC method calls
    /// - WebSocket messages
    /// - GraphQL queries/mutations
    ///
    /// # Associated Types
    ///
    /// - `Metrics`: The metrics produced by executing this action. Must be `Send` to cross
    ///   async task boundaries and be sent through channels to the Executor.
    ///
    /// # Async Execution
    ///
    /// The `execute` method is async to support non-blocking I/O operations. Each action
    /// execution produces metrics that are immediately sent to the Executor for aggregation
    /// and analysis.
    ///
    /// # Example Implementation
    ///
    /// ```ignore
    /// struct HttpAction {
    ///     method: String,
    ///     url: String,
    ///     headers: HashMap<String, String>,
    /// }
    ///
    /// impl Action for HttpAction {
    ///     type Metrics = HttpMetrics;
    ///
    ///     async fn execute(&self) -> HttpMetrics {
    ///         let start = Instant::now();
    ///         let response = http_client.request(&self).await;
    ///         HttpMetrics {
    ///             response_time: start.elapsed(),
    ///             status_code: response.status(),
    ///             error: response.error(),
    ///         }
    ///     }
    /// }
    /// ```
    pub trait Action {
        type Metrics;
        async fn execute(&self) -> Self::Metrics;
    }

    /// ActionGenerator trait - lazily generates actions for a VirtualUser
    ///
    /// Each VirtualUser owns a generator that produces actions on-demand, implementing Rust's
    /// `Iterator` trait for idiomatic lazy evaluation. Generators cover a partition of the test
    /// space (for multi-VUser scenarios) and are responsible for:
    ///
    /// - Generating test cases deterministically from a seed + partition
    /// - Supporting different test strategies (IPOG-C, security payloads, property-based, mutation)
    /// - Tracking position in the test space for reproducibility
    /// - Being thread-safe and serializable (for distributed execution)
    ///
    /// # Associated Types
    ///
    /// - `Action`: The type of action this generator produces. Must implement `Action` trait.
    ///
    /// # Iterator Pattern
    ///
    /// By extending `Iterator<Item = Self::Action>`, generators support:
    /// - Idiomatic `for` loops (though typically used with `next()`)
    /// - Standard iterator adapters (map, filter, etc.)
    /// - Integration with async `tokio::select!`
    ///
    /// # Test Space Partitioning
    ///
    /// When multiple VUsers run concurrently with the same generator configuration:
    /// - Each VUser receives a partitioned generator
    /// - Partition boundaries determined by VUser count and total test cases
    /// - Deterministic: `(seed, partition_start, case_index) → same action always`
    ///
    /// # Example Implementation
    ///
    /// ```ignore
    /// struct IPOGCGenerator {
    ///     seed: u64,
    ///     current_index: usize,
    ///     start_index: usize,      // partition start
    ///     end_index: usize,        // partition end
    ///     parameter_ranges: ParameterSpace,
    /// }
    ///
    /// impl ActionGenerator for IPOGCGenerator {
    ///     type Action = HttpAction;
    ///
    ///     fn next(&mut self) -> Option<HttpAction> {
    ///         if self.current_index >= self.end_index {
    ///             return None;
    ///         }
    ///         let action = self.generate_case(self.seed, self.current_index);
    ///         self.current_index += 1;
    ///         Some(action)
    ///     }
    ///
    ///     fn current_index(&self) -> usize {
    ///         self.current_index
    ///     }
    /// }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// `Send + Sync` bounds ensure generators can be shared safely across async tasks and threads.
    /// This is critical since:
    /// - VUser holds and mutates the generator
    /// - Generator may be sent across threads in distributed execution
    pub trait ActionGenerator: Send + Sync + Iterator<Item = Self::Action> {
        type Action: Action;

        /// Current position in the test space
        ///
        /// Used for:
        /// - Tracking progress
        /// - Debugging and logging
        /// - Determining when generator is exhausted (if total_cases known)
        fn current_index(&self) -> usize;
    }

    /// Signals sent from Executor to VirtualUser for control flow
    ///
    /// VUsers listen on a signal channel and respond to control signals from the Executor.
    /// This enables:
    /// - Granular stopping (stop specific VUsers based on metrics)
    /// - Future extensibility (pause, adjust rate, etc.)
    /// - Real-time response to executor decisions
    ///
    /// # Variants
    ///
    /// - `Stop`: Immediately terminate this VUser. The VUser will:
    ///   - Stop generating new actions
    ///   - Complete any in-flight actions
    ///   - Exit cleanly
    ///
    /// # Future Extensions
    ///
    /// Additional signals could be added:
    /// - `Pause`: Temporarily stop generating actions
    /// - `Resume`: Resume after pause
    /// - `AdjustRate(f64)`: Change requests per second dynamically
    /// - `CollectSnapshot`: Emit current metrics snapshot
    #[derive(Debug, Clone, Copy)]
    pub enum VUserSignal {
        Stop,
        // Future: Pause, AdjustRate(f64), etc.
    }

    /// VirtualUser - autonomous executor of actions with rate limiting and metrics collection
    ///
    /// A `VirtualUser` is a lightweight async task that:
    /// 1. Generates actions using an `ActionGenerator` (lazy, partitioned test space)
    /// 2. Executes actions at a controlled rate (rate limiting per-VUser)
    /// 3. Collects metrics from each action execution
    /// 4. Sends metrics immediately to the Executor for real-time analysis
    /// 5. Responds to control signals (Stop, etc.) from the Executor
    ///
    /// # Generic Parameter
    ///
    /// - `G: ActionGenerator` - The concrete generator type. Using a generic allows full
    ///   monomorphization: each VUser type is specialized for its generator at compile-time.
    ///   No runtime polymorphism or trait objects.
    ///
    /// # Ownership Model
    ///
    /// Each VirtualUser owns:
    /// - **id**: Unique identifier for logging and correlation
    /// - **generator**: Partitioned generator (deterministic test case source)
    /// - **metrics_tx**: Channel sender to Executor (for metrics)
    /// - **signal_rx**: Channel receiver from Executor (for control signals)
    /// - **rate_limiter**: Per-VUser rate limiting (no global bottleneck)
    ///
    /// # Execution Lifecycle
    ///
    /// A VirtualUser typically runs in a spawned Tokio task:
    ///
    /// ```ignore
    /// let vuser = VirtualUser::new(id, generator, metrics_tx, signal_rx, rps);
    /// tokio::spawn(vuser.run());  // Runs until Stop signal or generator exhausted
    /// ```
    ///
    /// During execution:
    /// 1. Wait for rate limit to elapse (via tokio::time::interval)
    /// 2. Check for stop signal (via tokio::select!)
    /// 3. Generate next action from generator
    /// 4. Execute action asynchronously
    /// 5. Send metrics immediately
    /// 6. Repeat until Stop signal or generator exhausted
    ///
    /// # Rate Limiting
    ///
    /// Rate limiting is decentralized (per-VUser):
    /// - Configured at creation time (requests_per_second)
    /// - Uses `tokio::time::interval` for fairness
    /// - Allows Executor to spawn many VUsers without bottlenecking
    /// - Can be extended to support dynamic adjustment (future)
    ///
    /// # Concurrency & Task Safety
    ///
    /// - Thread-safe: implements `Send` (necessary for Tokio spawning)
    /// - Generator is generic over concrete type (no trait objects)
    /// - Metrics sent through MPSC channel (thread-safe, non-blocking)
    /// - Signals received through MPSC channel (thread-safe, non-blocking)
    ///
    /// # Error Handling
    ///
    /// VirtualUser panics are caught by Executor's JoinSet:
    /// - `vuser_tasks.join_next()` will return `JoinError` if VUser panics
    /// - Executor logs and continues (doesn't abort other VUsers)
    /// - Metrics may be incomplete for that VUser
    pub struct VirtualUser<G: ActionGenerator> {
        id: VirtualUserId,
        generator: G,
        metrics_tx: tokio::sync::mpsc::Sender<<G::Action as Action>::Metrics>,
        signal_rx: tokio::sync::mpsc::Receiver<VUserSignal>,
        rate_limiter: crate::ratelimiter::RateLimiter,
    }

    impl<G: ActionGenerator> VirtualUser<G> {
        /// Create a new VirtualUser
        ///
        /// # Arguments
        ///
        /// - `id`: Unique identifier for this VUser. Converted to `VirtualUserId`.
        /// - `generator`: Action generator producing actions for this VUser.
        ///   Typically a partitioned generator covering a portion of test space.
        /// - `metrics_tx`: MPSC sender for sending action metrics to Executor.
        ///   Executor listens on corresponding `metrics_rx`.
        /// - `signal_rx`: MPSC receiver for control signals from Executor (e.g., Stop).
        ///   Executor holds corresponding `signal_tx`.
        /// - `requests_per_second`: Rate limit for this VUser (actions per second).
        ///   Enforced locally via tokio::time::interval.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let (metrics_tx, metrics_rx) = tokio::sync::mpsc::channel(100);
        /// let (signal_tx, signal_rx) = tokio::sync::mpsc::channel(1);
        ///
        /// let vuser = VirtualUser::new(
        ///     0,                          // VUser ID
        ///     my_generator,               // Partitioned generator
        ///     metrics_tx,
        ///     signal_rx,
        ///     10,                         // 10 requests/sec
        /// );
        /// ```
        pub fn new(
            id: impl Into<VirtualUserId>,
            generator: G,
            metrics_tx: tokio::sync::mpsc::Sender<<G::Action as Action>::Metrics>,
            signal_rx: tokio::sync::mpsc::Receiver<VUserSignal>,
            requests_per_second: usize,
        ) -> Self {
            let rate_limiter = crate::ratelimiter::RateLimiter::new(requests_per_second);
            VirtualUser {
                id: id.into(),
                generator,
                metrics_tx,
                signal_rx,
                rate_limiter,
            }
        }

        /// Run the VirtualUser - main execution loop
        ///
        /// Executes the following loop until stopped:
        /// 1. Wait for rate limit interval (tokio::time::interval)
        /// 2. Check for Stop signal from Executor
        /// 3. Generate next action from generator
        /// 4. Execute action asynchronously
        /// 5. Send metrics immediately via metrics_tx
        /// 6. Repeat
        ///
        /// Loop terminates when:
        /// - Executor sends `VUserSignal::Stop`
        /// - Generator exhausted (returns None)
        ///
        /// # Implementation Notes
        ///
        /// - Uses `tokio::select!` to interleave rate limiting with signal checking
        /// - Actions are executed sequentially (no parallelism within VUser)
        /// - Metrics sent immediately after each action (enables real-time monitoring)
        /// - Graceful shutdown: responds to Stop signal, doesn't abort in-flight actions
        ///
        /// # Panics
        ///
        /// If action execution panics, the VUser task will panic (caught by Executor's JoinSet).
        ///
        /// # Example Usage
        ///
        /// ```ignore
        /// let vuser = VirtualUser::new(...);
        /// tokio::spawn(vuser.run());  // Run in background task
        /// ```
        pub async fn run(mut self) {
            let rate_limit = 1.0; //TODO: change use the rate_limiter to regulate the rate of requests. Spawn reqeusts in a joinset
            let mut interval = tokio::time::interval(Duration::from_secs_f64(1.0 / rate_limit));
            loop {
                tokio::select! {
                    Some(signal) = self.signal_rx.recv() => {
                        match signal {
                            VUserSignal::Stop => break,
                        }
                    }
                    _ = interval.tick() => {
                        if let Some(action) = self.generator.next() {
                            let metrics = action.execute().await;
                            self.metrics_tx.send(metrics).await.ok();
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }
}
