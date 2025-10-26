# Trait Hierarchy and Relationships

This document describes all traits and their relationships in the fuzzer architecture.

## Core Traits

### TestPlanner

The foundation of test case generation.

```rust
pub trait TestPlanner: Iterator<Item = Case> {
    /// Create planner from configuration
    fn from_config(config: Configuration) -> Self
    where
        Self: Sized;

    /// Get algorithm name
    fn algorithm_name(&self) -> &'static str;

    /// Provide feedback to adaptive planners (optional)
    fn feedback(&mut self, _feedback: PlannerFeedback) {}
}
```

**Key Characteristics:**
- Extends `Iterator<Item = Case>` for lazy generation
- Stateful (can maintain internal state between `next()` calls)
- Optional feedback mechanism for adaptive planners
- No lifetime requirements (all owned data)

**Implementations:**
- `IpogPlanner` - In-Parameter-Order-General algorithm
- `IpogCPlanner` - IPOG with constraint handling
- `TwiseAdaptivePlanner` - Dynamic coverage adjustment
- `BoundaryPlanner` - Boundary value analysis
- `ManualPlanner` - Direct case specification

## Key Structs

### Case

A complete test case specification.

```rust
pub struct Case {
    pub id: String,
    pub name: String,
    pub request_template: RequestTemplate,
    pub executor_type: ExecutorType,
    pub assertions: Vec<Assertion>,
    pub stopping_conditions: Vec<StoppingCondition>,
}
```

**Purpose:**
- Declarative test specification
- Contains everything needed to execute and validate
- Immutable after creation (shared via cloning)

### RequestTemplate

HTTP request specification.

```rust
pub struct RequestTemplate {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub query_params: Vec<(String, String)>,
}
```

**Design Choices:**
- Simple owned data (String, Vec)
- No complex types (easy to serialize/deserialize)
- Body as `Option<Vec<u8>>` for binary support

## Enums

### ExecutorType

Defines how a test case is executed.

```rust
pub enum ExecutorType {
    /// Execute once
    SingleShot,

    /// Constant number of VUsers for duration
    ConstantVus {
        vus: usize,
        duration: Duration
    },

    /// Ramping VUsers through stages
    RampingVus {
        stages: Vec<Stage>
    },

    /// Constant arrival rate (open model)
    ConstantArrivalRate {
        rate: f64,          // requests per second
        duration: Duration,
        max_vus: usize      // upper bound
    },
}
```

**K6 Equivalents:**
- `SingleShot` → `shared-iterations`
- `ConstantVus` → `constant-vus`
- `RampingVus` → `ramping-vus`
- `ConstantArrivalRate` → `constant-arrival-rate`

### Assertion

Validation rules that must be satisfied.

```rust
pub enum Assertion {
    StatusCode(u16),
    P95ResponseTime(Duration),
    P99ResponseTime(Duration),
    SuccessRate(f64),        // 0.0 - 1.0
    ErrorRate(f64),          // 0.0 - 1.0
    MinRps(f64),
    MaxRps(f64),
    Custom {
        name: String,
        check: Box<dyn Fn(&ExecutorSnapshot) -> bool>,
    },
}
```

**Behavior:**
- Checked periodically by Executor
- Failure → test fails
- All must pass for test to succeed

### StoppingCondition

Early termination triggers.

```rust
pub enum StoppingCondition {
    FailureRate(f64),           // Stop if failure rate exceeds threshold
    ConsecutiveErrors(usize),   // Stop after N consecutive errors
    P95ResponseTime(Duration),  // Stop if p95 exceeds threshold
    P99ResponseTime(Duration),  // Stop if p99 exceeds threshold
    TotalRequests(usize),       // Stop after N requests
    Duration(Duration),         // Stop after time elapsed
}
```

**Behavior:**
- Checked periodically by Executor
- Match → graceful shutdown
- Does NOT fail the test (just stops early)

## Metrics Types

### VUserMetrics

Individual request metrics.

```rust
pub struct VUserMetrics {
    pub vuser_id: usize,
    pub timestamp: Instant,
    pub status_code: Option<u16>,
    pub response_time: Duration,
    pub success: bool,
    pub error: Option<String>,
}
```

**Flow:** VUser → Executor (via mpsc channel)

### ExecutorSnapshot

Aggregated case metrics.

```rust
pub struct ExecutorSnapshot {
    pub case_id: String,
    pub timestamp: Instant,
    pub total_requests: usize,
    pub successful: usize,
    pub failed: usize,
    pub current_rps: f64,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub active_vusers: usize,
}
```

**Flow:** Executor → Scheduler (via mpsc channel)

### GlobalSnapshot

Cross-case metrics.

```rust
pub struct GlobalSnapshot {
    pub timestamp: Instant,
    pub executors: Vec<ExecutorSnapshot>,
    pub total_requests: usize,
    pub total_successful: usize,
    pub total_failed: usize,
    pub overall_rps: f64,
}
```

**Flow:** Scheduler → MetricsHub → WebSocket/Storage

## Component Relationships

### Planner ←→ Scheduler

```rust
// Scheduler owns planner
struct Scheduler<P: TestPlanner> {
    planner: P,
    ...
}

// Scheduler polls cases
while let Some(case) = scheduler.planner.next() {
    // spawn executor
}

// Scheduler provides feedback (optional)
if let Some(feedback) = analyze_results(&snapshot) {
    scheduler.planner.feedback(feedback);
}
```

### Scheduler → Executor

```rust
// Scheduler spawns executors
let http_client = Arc::new(HttpClient::new());
let metrics_tx = scheduler.snapshot_sender();

let executor = Executor::new(case, http_client, metrics_tx);
tokio::spawn(async move {
    executor.run().await
});
```

### Executor → VUser

```rust
// Executor spawns VUsers
for i in 0..num_vusers {
    let vuser = VirtualUser::new(
        i,
        Arc::clone(&self.http_client),
        self.metrics_collector.metrics_tx.clone()
    );

    let handle = tokio::spawn(async move {
        vuser.execute_case(case).await
    });

    self.vuser_handles.push(handle);
}
```

### VUser → Target API

```rust
// VUser executes HTTP requests
impl VirtualUser {
    async fn execute_request(&self, template: &RequestTemplate) -> VUserMetrics {
        let request = match template.method {
            Method::Get => self.http_client.client.get(&template.url),
            // ... other methods
        };

        let result = request.send().await;
        // ... collect metrics
    }
}
```

## Ownership and Lifetimes

### Owned Data

All structs own their data (no lifetimes):
- `Case` owns `RequestTemplate`, `Vec<Assertion>`, etc.
- `VirtualUser` owns `id`, but shares `HttpClient` via `Arc`
- `Executor` owns `Case`, but shares `HttpClient` via `Arc`

### Shared Data (Arc)

Only one thing is shared: `HttpClient`

```rust
// Created once by Scheduler
let http_client = Arc::new(HttpClient::new());

// Cloned Arc (cheap, just pointer + refcount)
let executor1 = Executor::new(case1, Arc::clone(&http_client), ...);
let executor2 = Executor::new(case2, Arc::clone(&http_client), ...);

// Each VUser also gets Arc
let vuser = VirtualUser::new(id, Arc::clone(&http_client), ...);
```

**Why Arc?**
- reqwest::Client has internal connection pooling
- Sharing improves performance (reuses connections)
- No locks needed (Client is already thread-safe)

### No Shared Mutable State

VUsers have **zero shared mutable state**:
- Each VUser has its own `id`, `rate_limiter`
- Metrics sent via **owned message passing** (mpsc)
- No locks, no mutexes, no contention

## Channel Communication

### VUser → Executor

```rust
// Type: mpsc::Sender<VUserMetrics>
// Capacity: 10,000 messages
// Bounded to prevent memory explosion

vuser.metrics_tx.try_send(metrics)?;
```

### Executor → Scheduler

```rust
// Type: mpsc::Sender<ExecutorSnapshot>
// Capacity: 10,000 messages

executor.metrics_tx.send(snapshot).await?;
```

### Scheduler → MetricsHub

```rust
// Type: mpsc::Sender<ExecutorSnapshot>
// Owned by MetricsHub, cloned for each Executor

scheduler.metrics_hub.snapshot_tx.clone()
```

## Trait Implementations

### Iterator for TestPlanner

```rust
impl TestPlanner for IpogPlanner {
    fn from_config(config: Configuration) -> Self {
        // Initialize IPOG state
    }

    fn algorithm_name(&self) -> &'static str {
        "IPOG"
    }

    fn feedback(&mut self, feedback: PlannerFeedback) {
        // IPOG is not adaptive, ignore feedback
    }
}

impl Iterator for IpogPlanner {
    type Item = Case;

    fn next(&mut self) -> Option<Self::Item> {
        // Generate next test case
        // Returns None when all combinations covered
    }
}
```

### Clone for Case

```rust
// Case is Clone because:
// 1. Executor needs to share with VUsers
// 2. Feedback may reference case
// 3. No expensive data (mostly small Vecs)

impl Clone for Case {
    fn clone(&self) -> Self { ... }
}
```

## Type Safety Guarantees

### Compile-Time Guarantees

1. **No null pointers**: All `Option<T>` are explicit
2. **No data races**: No shared mutable state
3. **No lifetime issues**: All data is owned
4. **Channel type safety**: Can't send wrong message type

### Runtime Guarantees

1. **Bounded channels**: Prevents memory explosion
2. **Graceful degradation**: `try_send()` fails fast if channel full
3. **Timeout handling**: All HTTP requests have timeouts
4. **Error propagation**: `Result<T, E>` throughout

## Future Extensions

### Custom Planner Trait

```rust
pub trait CustomPlanner: TestPlanner {
    fn priority(&self, case: &Case) -> u8;
    fn should_skip(&self, case: &Case) -> bool;
}
```

### Custom Metrics

```rust
pub trait MetricsExporter {
    fn export(&self, snapshot: GlobalSnapshot) -> Result<(), Error>;
}

// Implementations:
// - PrometheusExporter
// - InfluxDBExporter
// - DatadogExporter
```

### Custom Validation

```rust
pub trait Validator {
    fn validate(&self, snapshot: &ExecutorSnapshot) -> ValidationResult;
}
```

## Summary

The trait system provides:

1. **Flexibility**: Trait-based design allows custom implementations
2. **Type Safety**: Rust's type system prevents common errors
3. **Performance**: Zero-cost abstractions, no runtime overhead
4. **Simplicity**: Minimal trait hierarchy, easy to understand
5. **Extensibility**: New planners, metrics, validators can be added

The key insight is that **data structures are more important than traits** in this architecture. Most types are concrete structs, with traits used only where flexibility is needed (TestPlanner).
