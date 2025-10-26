# Elerem Fuzzer

High-performance API testing and fuzzing framework inspired by K6, built in Rust with advanced test generation algorithms.

## Architecture Overview

The fuzzer is built with 5 distinct layers, each with clear responsibilities:

```
┌─────────────────────────────────────────────────────────────────┐
│                         1. CONFIGURATION                          │
│  User input: JSON, YAML, logs, manual entry                      │
│  • Base URLs, auth, scenarios                                    │
│  • High-level test objectives                                    │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                         2. PLANNER                                │
│  Test case generation with algorithms:                           │
│  • IPOG-C: T-wise testing with constraints                       │
│  • T-wise Adaptive: Dynamic coverage                             │
│  • Boundary analysis, edge cases                                 │
│  Iterator-based lazy generation                                  │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         │ Iterator<Item = Case>
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                         3. SCHEDULER                              │
│  Test orchestration & metrics hub:                               │
│  • Polls cases from Planner                                      │
│  • Manages parallel/sequential execution                         │
│  • Owns MetricsHub (Stream + Recorder)                           │
│  • Provides feedback to adaptive planners                        │
│  • WebSocket communication to website                            │
└─────┬──────────────────────────────┬────────────────────────────┘
      │                              │
      │ spawn Executor               │ MetricsHub
      │                              │ ├─ MetricsStream (WebSocket, 60s history)
      ▼                              │ └─ MetricsRecorder (persistent storage)
┌─────────────────────────────────┐  │
│       4. EXECUTOR               │  │
│  Manages single test case:      │  │
│  • Spawns N Virtual Users       │  │
│  • Enforces assertions          │  │
│  • Checks stopping conditions   │  │
│  • Aggregates VUser metrics     │  │
│  • Sends snapshots upstream ────┼──┘
└─────┬───────────────────────────┘
      │
      │ spawn VUser (x N)
      ▼
┌─────────────────────────────────┐
│       5. VIRTUAL USERS          │
│  Dumb workers:                  │
│  • Share-nothing architecture   │
│  • Shared HttpClient (Arc)      │
│  • Per-VUser rate limiting      │
│  • Sends metrics upstream       │
└─────────────────────────────────┘
```

## Data Flow

### Forward Flow: Configuration → Execution

```
Config ──→ Planner ──→ Scheduler ──→ Executor ──→ VirtualUser
                                         │              │
                                         │              │ HTTP
                                         │              ▼
                                         │           [API]
                                         │              │
                                         │              │ Metrics
                                         │◀─────────────┘
```

### Metrics Flow: VUser → Website

```
VirtualUser ──→ Executor ──→ Scheduler ──→ MetricsHub ──┬──→ MetricsStream ──→ WebSocket ──→ Website
                                                         │
                                                         └──→ MetricsRecorder ──→ Storage
```

### Feedback Loop: Adaptive Planning

```
                    ┌─────────────────┐
                    │   PlannerFeedback│
                    └────────▲─────────┘
                             │
Planner ──→ Scheduler ───────┘
   │            │
   │ cases      │ results
   └────→───────┘
```

## Core Components

### 1. Configuration

User-facing configuration in any format (JSON, YAML, etc.):

```rust
pub struct Config {
    pub base_url: String,
    pub auth: AuthConfig,
    pub scenarios: Vec<Scenario>,
}
```

### 2. Planner

Generates test cases using advanced algorithms. Implements `Iterator<Item = Case>` for lazy generation:

```rust
pub trait TestPlanner: Iterator<Item = Case> {
    fn update(&mut self, feedback: PlannerFeedback);
}

pub struct Case {
    pub id: String,
    pub request_template: RequestTemplate,
    pub executor_type: ExecutorType,
    pub assertions: Vec<Assertion>,
    pub stopping_conditions: Vec<StoppingCondition>,
}
```

**Test Generation Algorithms:**
- **IPOG (In-Parameter-Order-General)**: Efficient t-way combinatorial testing
- **IPOG-C**: IPOG with constraint handling for complex systems
- **T-wise Adaptive**: Dynamically adjusts coverage based on runtime behavior
- **Boundary Value Analysis**: Edge cases and boundary testing
- **Constraint-Based**: Custom rules and relationships

### 3. Scheduler

Orchestrates test execution and manages metrics infrastructure:

```rust
pub struct Scheduler<P: TestPlanner> {
    planner: P,
    metrics_hub: Arc<MetricsHub>,
    config: SchedulerConfig,
}
```

**Responsibilities:**
- Poll cases from Planner (lazy iteration)
- Spawn Executors for each case
- Manage parallel vs sequential execution
- Own MetricsHub (coordinate streaming and recording)
- Provide feedback to adaptive planners
- Global stopping condition enforcement

### 4. Executor

Manages a single test case with N virtual users:

```rust
pub struct Executor {
    case: Case,
    http_client: Arc<HttpClient>,
    metrics_collector: MetricsCollector,
    vuser_handles: Vec<JoinHandle<()>>,
    metrics_tx: mpsc::Sender<ExecutorSnapshot>,
}
```

**Responsibilities:**
- Spawn and manage VUsers based on ExecutorType
- Enforce case-level assertions
- Check stopping conditions
- Aggregate VUser metrics
- Calculate percentiles (p50, p95, p99)
- Send snapshots to Scheduler

**Executor Types:**
- `SingleShot { count }`: One-time execution with N VUsers
- `ConstantVus { vus, duration }`: Constant load
- `RampingVus { stages }`: Gradual load increase/decrease

### 5. Virtual Users

Dumb workers that execute requests:

```rust
pub struct VirtualUser {
    id: usize,
    http_client: Arc<HttpClient>,
    metrics_tx: mpsc::Sender<VUserMetrics>,
    rate_limiter: Option<RateLimiter>,
}
```

**Characteristics:**
- Share-nothing architecture (independent execution)
- Shared `HttpClient` via Arc (connection pooling handled by reqwest)
- Per-VUser or global rate limiting
- Sends metrics upstream after each request

## Metrics Architecture

### MetricsHub

Central coordinator for metrics streaming and recording:

```rust
pub struct MetricsHub {
    snapshot_tx: mpsc::Sender<ExecutorSnapshot>,
    stream: MetricsStream,
    recorder: MetricsRecorder,
    global_metrics: Arc<RwLock<GlobalMetrics>>,
}
```

### MetricsStream

Real-time streaming to WebSocket clients:
- Maintains recent history (60 seconds)
- Broadcasts to connected clients
- Low-latency updates (sub-second)

### MetricsRecorder

Persistent storage of metrics:
- Buffered writes for performance
- Multiple storage backends:
  - Database (SQLite, PostgreSQL)
  - File (JSONL, Parquet)
  - Memory (for testing)
  - None (disable recording)

## Validation System

### Assertions

Rules that **must** be satisfied for the test to pass:

```rust
pub enum Assertion {
    StatusCode(u16),                    // Expected status code
    P95ResponseTime(Duration),          // 95th percentile max
    SuccessRate(f64),                   // Minimum success rate (0.0-1.0)
    ErrorRate(f64),                     // Maximum error rate (0.0-1.0)
    MinRps(f64),                        // Minimum requests/second
    MaxRps(f64),                        // Maximum requests/second
}
```

### Stopping Conditions

Conditions that trigger **early termination**:

```rust
pub enum StoppingCondition {
    FailureRate(f64),                   // Stop if failure rate exceeds threshold
    ConsecutiveErrors(usize),           // Stop after N consecutive errors
    P95ResponseTime(Duration),          // Stop if p95 exceeds threshold
    TotalRequests(usize),               // Stop after N total requests
}
```

**Ownership:**
- **Case**: Defines rules (declarative)
- **Executor**: Enforces rules (runtime checking)
- **Scheduler**: Handles global conditions (across all executors)

## Example Usage

```rust
use fuzzer::{Config, Planner, Scheduler};

#[tokio::main]
async fn main() {
    // 1. Load configuration
    let config = Config::from_file("test-config.json").await?;

    // 2. Create planner with IPOG algorithm
    let planner = IpogPlanner::new(config);

    // 3. Create scheduler
    let scheduler = Scheduler::new(planner, SchedulerConfig {
        enable_streaming: true,
        enable_recording: true,
        storage: StorageConfig::Database("results.db".into()),
        parallel_execution: true,
    });

    // 4. Run tests
    scheduler.run().await?;
}
```

## Test Case Example

```rust
Case {
    id: "login-load-test".into(),
    request_template: RequestTemplate {
        method: Method::Post,
        url: "https://api.example.com/login".into(),
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: Some(r#"{"email":"test@example.com","password":"test123"}"#.into()),
    },
    executor_type: ExecutorType::RampingVus {
        stages: vec![
            Stage { duration: Duration::from_secs(10), target_vus: 10 },
            Stage { duration: Duration::from_secs(30), target_vus: 100 },
            Stage { duration: Duration::from_secs(10), target_vus: 0 },
        ],
    },
    assertions: vec![
        Assertion::StatusCode(200),
        Assertion::P95ResponseTime(Duration::from_millis(200)),
        Assertion::SuccessRate(0.99),
    ],
    stopping_conditions: vec![
        StoppingCondition::FailureRate(0.15),
        StoppingCondition::ConsecutiveErrors(50),
    ],
}
```

## Key Design Decisions

### Share-Nothing VUsers

Each VUser is independent with its own state:
- No shared mutable state between VUsers
- Simplifies reasoning about concurrency
- Enables true parallelism without locks

### Shared HttpClient

`reqwest::Client` is shared via `Arc`:
- Connection pooling handled internally by reqwest
- Efficient resource usage
- No performance penalty from sharing

### Iterator-Based Planner

Cases are generated lazily:
- Memory efficient (no upfront generation)
- Adaptive planners can adjust based on feedback
- Supports infinite test generation

### Three-Level Metrics Hierarchy

```
VUser → Executor → Scheduler (MetricsHub)
```

- VUser: Individual request metrics
- Executor: Aggregated case metrics (percentiles, rates)
- Scheduler: Global metrics across all cases

### Validation Ownership

- **Case**: Declarative rules (what to check)
- **Executor**: Runtime enforcement (when to check)
- **Scheduler**: Global coordination (stop all)

## Performance Characteristics

- **High throughput**: Async I/O with tokio
- **Low latency**: Lock-free VUser execution
- **Memory efficient**: Lazy case generation, bounded buffers
- **Scalable**: Horizontal scaling via multiple executors

## Future Enhancements

- [ ] Distributed execution across multiple nodes
- [ ] Custom JavaScript/Wasm scripting for complex scenarios
- [ ] Machine learning-based adaptive test generation
- [ ] Browser-based testing (Playwright integration)
- [ ] GraphQL and gRPC protocol support
- [ ] Custom plugin system for metrics exporters

## License

MIT
