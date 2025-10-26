# Elerem Fuzzer

High-performance API testing and fuzzing framework inspired by K6, built in Rust with advanced test generation algorit│  │  struct VirtualUser {                                                                 │ │
│  │    id: usize,                              // VUser identifier                       │ │
│  │    http_client: reqwest::Client,           // Shared (Client uses Arc internally)    │ │
│  │    metrics_tx: mpsc::Sender<VUserMetrics>, // Channel to send metrics to Executor    │ │
│  │    rate_limiter: RateLimiter,              // Always present (0.0 = unlimited)       │ │
│  │    parallel_requests: usize,               // 1 = sequential, N = parallel           │ │
│  │  }                                                                                    │ │---

## Complete Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    USER / WEBSITE                                           │
│  Provides JSON/YAML config → Receives real-time metrics via WebSocket                      │
└────────────────────────────────┬──────────────────────────────────▲───────────────────────┘
                                 │                                  │
                                 │ Config                           │ GlobalSnapshot
                                 │ {base_url, auth, scenarios}      │ (WebSocket stream)
                                 ▼                                  │
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              LAYER 1: CONFIGURATION                                         │
│  ┌───────────────────────────────────────────────────────────────────────────────────────┐ │
│  │  Parse and validate user input                                                        │ │
│  │  • JSON/YAML/manual entry                                                             │ │
│  │  • Extract parameters for test generation                                             │ │
│  │  • Validate auth, URLs, scenarios                                                     │ │
│  └───────────────────────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────┬────────────────────────────────────────────────────────────┘
                                 │ Configuration
                                 ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              LAYER 2: PLANNER                                               │
│  ┌───────────────────────────────────────────────────────────────────────────────────────┐ │
│  │  trait TestPlanner: Iterator<Item = Case>                                            │ │
│  │                                                                                       │ │
│  │  Algorithms (pick one):                                                              │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐  ┌────────────────┐│ │
│  │  │ IPOG            │  │ IPOG-C          │  │ T-wise Adaptive  │  │ Boundary       ││ │
│  │  │ Combinatorial   │  │ With constraints│  │ Dynamic coverage │  │ Edge cases     ││ │
│  │  │ testing         │  │ handling        │  │ adjustment       │  │ analysis       ││ │
│  │  └─────────────────┘  └─────────────────┘  └──────────────────┘  └────────────────┘│ │
│  │                                                                                       │ │
│  │  fn next() -> Option<Case>     ◄───────────────┐                                     │ │
│  │  fn feedback(PlannerFeedback)  ◄───────────────┼─ Adaptive feedback loop            │ │
│  │                                                 │   (adjust tests based on results)  │ │
│  │  Generates:                                     │                                     │ │
│  │  Case {                                         │                                     │ │
│  │    id: String,                                  │                                     │ │
│  │    request_template: RequestTemplate,           │                                     │ │
│  │    executor_type: ExecutorType,                 │                                     │ │
│  │    assertions: Vec<Assertion>,                  │                                     │ │
│  │    stopping_conditions: Vec<StoppingCondition>  │                                     │ │
│  │  }                                              │                                     │ │
│  └───────────────────────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────┬────────────────────────────────────────────────────────────┘
                                 │ Iterator<Item = Case>
                                 │ (lazy generation, one at a time)
                                 ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              LAYER 3: SCHEDULER                                             │
│  ┌───────────────────────────────────────────────────────────────────────────────────────┐ │
│  │  Orchestrates everything                                                              │ │
│  │                                                                                       │ │
│  │  Main loop:                                                                           │ │
│  │  1. Poll planner.next() → get Case                                                   │ │
│  │  2. Create HttpClient (shared connection pool)                                  │ │
│  │  3. Spawn tokio task: Executor::new(case, http_client, metrics_tx).run()            │ │
│  │  4. Receive ExecutorSnapshot via mpsc channel                                        │ │
│  │  5. Update MetricsHub (aggregate, stream, record)                                    │ │
│  │  6. Check global stopping conditions                                                 │ │
│  │  7. Send feedback to adaptive planners ──────────────────────────────────────────────┼─┐│
│  │                                                                                       │ ││
│  │  Owns:                                                                                │ ││
│  │  ┌──────────────────────────────────────────────────────────────────────┐            │ ││
│  │  │ MetricsHub                                                            │            │ ││
│  │  │  ├─ MetricsStream:  WebSocket server, broadcasts GlobalSnapshot      │───────────┼─┼┤
│  │  │  │                  Keeps 60s circular buffer (120 snapshots @ 2/s)   │           │ ││
│  │  │  │                                                                     │           │ ││
│  │  │  └─ MetricsRecorder: Persistent storage (DB/File/Memory)             │           │ ││
│  │  │                      Buffered writes (flush every 100 snapshots)     │           │ ││
│  │  └──────────────────────────────────────────────────────────────────────┘            │ ││
│  └───────────────────────────────────────────────────────────────────────────────────────┘ ││
└────────────────────────────────┬────────────────────────────────────────────────────────────┘│
                                 │ spawn multiple executors                                    │
                                 │ (parallel or sequential based on config)                    │
                                 ▼                                                             │
┌─────────────────────────────────────────────────────────────────────────────────────────────┤
│                              LAYER 4: EXECUTOR                                              │
│  ┌───────────────────────────────────────────────────────────────────────────────────────┐ │
│  │  Manages single Case with N VirtualUsers                                              │ │
│  │                                                                                       │ │
│  │  Initialization:                                                                      │ │
│  │  1. Parse ExecutorType (SingleShot/ConstantVus/RampingVus/ConstantArrivalRate)      │ │
│  │  2. Spawn N VirtualUsers based on type                                               │ │
│  │  3. Create mpsc channel for VUserMetrics                                             │ │
│  │                                                                                       │ │
│  │  Execution Loop (every 1 second):                                                    │ │
│  │  1. Collect VUserMetrics from channel                                                │ │
│  │  2. Aggregate:                                                                        │ │
│  │     • Count: total_requests, successful, failed                                      │ │
│  │     • Calculate: current RPS                                                         │ │
│  │     • Store response_times in VecDeque (for percentiles)                             │ │
│  │     • Calculate p50, p95, p99                                                        │ │
│  │  3. Check Assertions (defined in Case):                                              │ │
│  │     • StatusCode(200) → check most common status                                     │ │
│  │     • P95ResponseTime(100ms) → calculate_percentile(0.95) <= 100ms                  │ │
│  │     • SuccessRate(0.99) → (successful / total) >= 0.99                              │ │
│  │     • ErrorRate(0.05) → (failed / total) <= 0.05                                    │ │
│  │     → If any fail: return Err("Assertion failed")                                    │ │
│  │  4. Check StoppingConditions (defined in Case):                                      │ │
│  │     • FailureRate(0.15) → (failed / total) >= 0.15                                  │ │
│  │     • ConsecutiveErrors(50) → consecutive_errors >= 50                              │ │
│  │     • P95ResponseTime(5s) → p95 > 5s                                                │ │
│  │     → If any match: return Ok(()) (graceful stop)                                    │ │
│  │  5. Send ExecutorSnapshot to Scheduler via metrics_tx                               │ │
│  │                                                                                       │ │
│  │  ExecutorSnapshot sent to Scheduler:                                                 │ │
│  │  {                                                                                    │ │
│  │    case_id: "login-test",                                                            │ │
│  │    timestamp: Instant::now(),                                                        │ │
│  │    total_requests: 1523,                                                             │ │
│  │    successful: 1520,                                                                 │ │
│  │    failed: 3,                                                                        │ │
│  │    current_rps: 25.4,                                                                │ │
│  │    p50: 45ms, p95: 87ms, p99: 152ms,                                                │ │
│  │    active_vusers: 10                                                                 │ │
│  │  }                                                                                    │ │
│  └───────────────────────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────┬────────────────────────────────────────────────────────────┘
                                 │ spawn N VirtualUsers
                                 │ (tokio tasks)
                                 ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              LAYER 5: VIRTUAL USERS                                         │
│  ┌───────────────────────────────────────────────────────────────────────────────────────┐  │
│  │  Dumb workers executing HTTP requests                                                 │  │
│  │                                                                                       │  │
│  │  struct VirtualUser {                                                                 │  │
│  │    id: usize,                              // VUser identifier                        │  │
│  │    http_client: HttpClient,           // connection pooling                           │  │
│  │    metrics_tx: mpsc::Sender<VUserMetrics>, // Channel to send metrics to Executor     │  │
│  │    rate_limiter: RateLimiter       // Per-VUser rate limiting                         │  │
│  │  }                                                                                    │  │
│  │                                                                                       │  │
│  │  Execution Loop (infinite until stopped):                                             │  │
│  │  ┌──────────────────────────────────────────────────────────────────────────────┐   │ │
│  │  │ 1. Apply rate limiter (if configured)                                        │   │ │
│  │  │    → Wait if needed to maintain requests_per_second                          │   │ │
│  │  │                                                                               │   │ │
│  │  │ 2. Build HTTP request from RequestTemplate:                                  │   │ │
│  │  │    // Use efficient to_request_builder() method                             │   │ │
│  │  │    let request = template.to_request_builder(&http_client);                 │   │ │
│  │  │    // Handles: method, headers, body, query_params                          │   │ │
│  │  │                                                                               │   │ │
│  │  │ 3. Execute request:                                                           │   │ │
│  │  │    let start = Instant::now();                                               │   │ │
│  │  │    let result = request.send().await;  ────────────────────────────────┐    │   │ │
│  │  │    let elapsed = start.elapsed();                                      │    │   │ │
│  │  │                                                                          │    │   │ │
│  │  │ 4. Collect metrics:                                                      │    │   │ │
│  │  │    let metrics = VUserMetrics {                                          │    │   │ │
│  │  │      vuser_id: self.id,                                                  │    │   │ │
│  │  │      timestamp: start,                                                   │    │   │ │
│  │  │      status_code: result.ok().map(|r| r.status().as_u16()),            │    │   │ │
│  │  │      response_time: elapsed,                                             │    │   │ │
│  │  │      success: result.is_ok() && status.is_success(),                    │    │   │ │
│  │  │      error: result.err().map(|e| e.to_string())                         │    │   │ │
│  │  │    };                                                                     │    │   │ │
│  │  │                                                                          │    │   │ │
│  │  │ 5. Send metrics upstream:                                                │    │   │ │
│  │  │    metrics_tx.try_send(metrics)?;                                        │    │   │ │
│  │  │    → If channel full or closed, stop VUser                               │    │   │ │
│  │  │                                                                          │    │   │ │
│  │  │ 6. Loop back to step 1                                                   │    │   │ │
│  │  └──────────────────────────────────────────────────────────────────────────────┘   │ │
│  │                                                                          │            │ │
│  │  Key Design:                                                             │            │ │
│  │  • SHARE-NOTHING: Each VUser is completely independent                   │            │ │
│  │  • No shared mutable state between VUsers                                │            │ │
│  │  • HttpClient cloned (reqwest::Client uses Arc internally)              │            │ │
│  │  • RateLimiter always present (0.0 = unlimited, avoids branches)        │            │ │
│  │  • Parallel requests: Each VUser can spawn N concurrent requests        │            │ │
│  │  • Metrics sent via owned message passing (mpsc channel)                 │            │ │
│  │  • No locks, no mutexes, no contention                                   │            │ │
│  └───────────────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┬───────────────────────────────┘
                                                              │ HTTP
                                                              ▼
                                                    ┌──────────────────┐
                                                    │   TARGET API     │
                                                    │  (Under Test)    │
                                                    └──────────────────┘
```

---

## Data Flow Details

### Metrics Flow (Bottom-Up)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          METRICS AGGREGATION HIERARCHY                                  │
└─────────────────────────────────────────────────────────────────────────────────────────┘

LEVEL 1: VUserMetrics (Individual Request)
┌──────────────────────────────────────────────────────────────────┐
│ VirtualUser #1                VirtualUser #2    ...    VUser #N  │
│      │                              │                      │      │
│      │ VUserMetrics                 │                      │      │
│      │ {                            │                      │      │
│      │   vuser_id: 1,               │                      │      │
│      │   timestamp: Instant,        │                      │      │
│      │   status_code: Some(200),    │                      │      │
│      │   response_time: 45ms,       │                      │      │
│      │   success: true,             │                      │      │
│      │   error: None                │                      │      │
│      │ }                            │                      │      │
│      └──────────────────────────────┴──────────────────────┘      │
│                                │                                  │
│                    mpsc::Sender<VUserMetrics>                     │
│                    Capacity: 10,000 messages                      │
│                    Bounded to prevent memory explosion            │
│                                │                                  │
└────────────────────────────────┼──────────────────────────────────┘
                                 ▼

LEVEL 2: ExecutorSnapshot (Per-Case Aggregation)
┌──────────────────────────────────────────────────────────────────┐
│ Executor (Case "login-test")                                     │
│      │                                                            │
│      │ Aggregation every 1 second:                               │
│      │ • Collect all VUserMetrics from channel                   │
│      │ • Count: total_requests, successful, failed               │
│      │ • Calculate: current_rps = requests / elapsed_time        │
│      │ • Store: response_times in VecDeque<Duration>             │
│      │ • Calculate: p50, p95, p99 from sorted response_times     │
│      │                                                            │
│      │ ExecutorSnapshot {                                        │
│      │   case_id: "login-test",                                  │
│      │   timestamp: Instant::now(),                              │
│      │   total_requests: 1523,                                   │
│      │   successful: 1520,                                       │
│      │   failed: 3,                                              │
│      │   current_rps: 25.4,                                      │
│      │   p50: 45ms,                                              │
│      │   p95: 87ms,                                              │
│      │   p99: 152ms,                                             │
│      │   active_vusers: 10                                       │
│      │ }                                                          │
│      └────────────────────────────────────────────────────────────┤
│                                │                                  │
│                    mpsc::Sender<ExecutorSnapshot>                 │
│                    Capacity: 10,000 messages                      │
│                                │                                  │
└────────────────────────────────┼──────────────────────────────────┘
                                 ▼

LEVEL 3: GlobalSnapshot (Cross-Case Aggregation)
┌──────────────────────────────────────────────────────────────────┐
│ Scheduler (MetricsHub)                                           │
│      │                                                            │
│      │ Aggregation:                                              │
│      │ • Receive ExecutorSnapshots from all executors            │
│      │ • Store in Vec<ExecutorSnapshot>                          │
│      │ • Calculate global totals:                                │
│      │   - total_requests = sum(executor.total_requests)         │
│      │   - total_successful = sum(executor.successful)           │
│      │   - total_failed = sum(executor.failed)                   │
│      │   - overall_rps = sum(executor.current_rps)               │
│      │                                                            │
│      │ GlobalSnapshot {                                          │
│      │   timestamp: Instant::now(),                              │
│      │   executors: [                                            │
│      │     ExecutorSnapshot { case_id: "login-test", ... },      │
│      │     ExecutorSnapshot { case_id: "search-test", ... },     │
│      │     ...                                                   │
│      │   ],                                                       │
│      │   total_requests: 5234,                                   │
│      │   total_successful: 5201,                                 │
│      │   total_failed: 33,                                       │
│      │   overall_rps: 87.2                                       │
│      │ }                                                          │
│      │                                                            │
│      ├──────────────────────────────────────────────────┐        │
│      │                                                   │        │
│      ▼                                                   ▼        │
│ MetricsStream                               MetricsRecorder      │
│ • Broadcast to WebSocket clients            • Buffer snapshots   │
│ • Keep 60s circular buffer                  • Flush every 100    │
│ • Send GlobalSnapshot @ 2/sec               • Write to:          │
│                                              - Database (SQLite)  │
│                                              - File (JSONL)       │
│                                              - Memory (testing)   │
└──────────────────────────────────────────────────────────────────┘
```

### Feedback Loop (Adaptive Planning)

```
┌───────────────────────────────────────────────────────────────────────────┐
│                         ADAPTIVE FEEDBACK CYCLE                           │
└───────────────────────────────────────────────────────────────────────────┘

  Planner                     Scheduler                    Executor
     │                            │                            │
     │ 1. next() → Case           │                            │
     ├───────────────────────────►│                            │
     │                            │                            │
     │                            │ 2. spawn(case)             │
     │                            ├───────────────────────────►│
     │                            │                            │
     │                            │                            │ 3. Execute
     │                            │                            │    with N VUsers
     │                            │                            │
     │                            │                            │ 4. Collect metrics
     │                            │                            │    & validate
     │                            │                            │
     │                            │ 5. ExecutorSnapshot        │
     │                            │◄───────────────────────────┤
     │                            │    {success/failure,       │
     │                            │     failure_rate,          │
     │                            │     p95_response_time}     │
     │                            │                            │
     │ 6. feedback()              │                            │
     │    PlannerFeedback {       │                            │
     │      case_id,              │                            │
     │      success: bool,        │                            │
     │      failure_rate: 0.02,   │                            │
     │      response_time_p95     │                            │
     │    }                       │                            │
     │◄───────────────────────────┤                            │
     │                            │                            │
     │ 7. Adjust strategy:        │                            │
     │    • If high failure:      │                            │
     │      generate similar      │                            │
     │      cases to explore      │                            │
     │    • If successful:        │                            │
     │      reduce coverage       │                            │
     │      in that area          │                            │
     │                            │                            │
     │ 8. next() → Adjusted Case  │                            │
     ├───────────────────────────►│                            │
     │                            │                            │
     └────────────────────────────┴────────────────────────────┘

Example: T-wise Adaptive Planner
• Initial: Generate all 2-way parameter combinations (IPOG)
• Feedback: Case with {auth=oauth, endpoint=/users} failed 15%
• Adjustment: Generate more 3-way combinations involving oauth + /users
• Result: Increased coverage in problematic areas, reduced elsewhere
```

### Validation Flow (Assertions & Stopping Conditions)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            VALIDATION OWNERSHIP                              │
└──────────────────────────────────────────────────────────────────────────────┘

CASE (Declarative Rules)
┌────────────────────────────────────────────────────────────┐
│ Case {                                                     │
│   assertions: [                                            │
│     Assertion::StatusCode(200),                            │
│     Assertion::P95ResponseTime(Duration::from_millis(100)),│
│     Assertion::SuccessRate(0.99),  // 99%                  │
│     Assertion::ErrorRate(0.01),    // 1%                   │
│   ],                                                        │
│   stopping_conditions: [                                   │
│     StoppingCondition::FailureRate(0.15),  // Stop @ 15%   │
│     StoppingCondition::ConsecutiveErrors(50),              │
│     StoppingCondition::P95ResponseTime(Duration::from_secs(5)),│
│     StoppingCondition::TotalRequests(10_000),              │
│   ],                                                        │
│ }                                                           │
└────────────────────────────────────────────────────────────┘
           │
           │ Case passed to Executor
           ▼
EXECUTOR (Runtime Enforcement)
┌────────────────────────────────────────────────────────────┐
│ Every 1 second:                                            │
│                                                             │
│ 1. Check all assertions:                                   │
│    for assertion in case.assertions {                      │
│      match assertion {                                     │
│        StatusCode(200) => {                                │
│          let most_common = status_codes.max_by_key();      │
│          if most_common != 200 {                           │
│            return Err("StatusCode assertion failed");      │
│          }                                                 │
│        }                                                   │
│        P95ResponseTime(max) => {                           │
│          let p95 = calculate_percentile(&times, 0.95);     │
│          if p95 > max {                                    │
│            return Err("P95 assertion failed");             │
│          }                                                 │
│        }                                                   │
│        SuccessRate(min) => {                               │
│          if (successful / total) < min {                   │
│            return Err("SuccessRate assertion failed");     │
│          }                                                 │
│        }                                                   │
│      }                                                     │
│    }                                                       │
│                                                             │
│ 2. Check stopping conditions:                              │
│    for condition in case.stopping_conditions {             │
│      match condition {                                     │
│        FailureRate(threshold) => {                         │
│          if (failed / total) >= threshold {                │
│            return Ok(()); // Graceful stop                 │
│          }                                                 │
│        }                                                   │
│        ConsecutiveErrors(max) => {                         │
│          if consecutive_errors >= max {                    │
│            return Ok(()); // Graceful stop                 │
│          }                                                 │
│        }                                                   │
│      }                                                     │
│    }                                                       │
│                                                             │
│ Result:                                                     │
│ • Assertion fails → Err("...") → Test FAILED              │
│ • Stopping condition → Ok(()) → Test stopped early        │
│ • All pass → Continue executing                            │
└────────────────────────────────────────────────────────────┘
           │
           │ ExecutorSnapshot with status
           ▼
SCHEDULER (Global Coordination)
┌────────────────────────────────────────────────────────────┐
│ When Executor returns:                                     │
│                                                             │
│ match executor_result {                                    │
│   Err(e) => {                                              │
│     log_error!("Test failed: {}", e);                      │
│     // Optionally stop all executors                       │
│     // Send failure notification                           │
│   }                                                        │
│   Ok(()) => {                                              │
│     log_info!("Test stopped early (stopping condition)");  │
│     // Continue with next case                             │
│   }                                                        │
│ }                                                           │
│                                                             │
│ Global stopping conditions:                                │
│ • Total failure rate across ALL cases > threshold          │
│ • Maximum test duration exceeded                           │
│ • User cancellation                                        │
└────────────────────────────────────────────────────────────┘
```

---

## Concurrency Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           TASK HIERARCHY                                │
└─────────────────────────────────────────────────────────────────────────┘

Scheduler (Single tokio task)
│
│  Responsibilities:
│  • Poll planner.next() sequentially
│  • Spawn executor tasks
│  • Receive metrics via mpsc channel
│  • Coordinate MetricsHub
│  • No locks, no shared mutable state
│
├─── Executor 1 (tokio task)
│    │  Case: "login-test"
│    │  Manages: 10 VUsers
│    │
│    ├─── VUser 1 (tokio task) ─────┐
│    ├─── VUser 2 (tokio task) ─────┤
│    ├─── VUser 3 (tokio task) ─────┤ Independent
│    ├─── VUser 4 (tokio task) ─────┤ No shared
│    ├─── VUser 5 (tokio task) ─────┤ mutable state
│    ├─── VUser 6 (tokio task) ─────┤ No locks
│    ├─── VUser 7 (tokio task) ─────┤ Share-nothing
│    ├─── VUser 8 (tokio task) ─────┤ architecture
│    ├─── VUser 9 (tokio task) ─────┤
│    └─── VUser 10 (tokio task) ────┘
│         │
│         └─→ All share HttpClient (read-only, thread-safe)
│
├─── Executor 2 (tokio task)
│    │  Case: "search-test"
│    │  Manages: 50 VUsers
│    │
│    ├─── VUser 1..50 (tokio tasks)
│
├─── Executor 3 (tokio task)
│    │  Case: "checkout-test"
│    │  Manages: 5 VUsers
│    │
│    └─── VUser 1..5 (tokio tasks)
│
└─── MetricsHub Task (tokio task)
     │
     ├─── MetricsStream Task (WebSocket server)
     │    • Accept connections
     │    • Broadcast GlobalSnapshot every 500ms
     │
     └─── MetricsRecorder Task (Background flusher)
          • Flush buffer every 100 snapshots
          • Write to storage (DB/File)

Communication:
• VUser → Executor: mpsc::Sender<VUserMetrics> (bounded, 10k capacity)
• Executor → Scheduler: mpsc::Sender<ExecutorSnapshot> (bounded, 10k capacity)
• Scheduler → MetricsHub: Direct method calls (owns MetricsHub)

No Locks Required Because:
• Each VUser has its own state (id, rate_limiter)
• Metrics sent via owned message passing (mpsc channels)
• reqwest::Client is thread-safe (uses Arc internally)
• No shared mutable state anywhere in the system
```

---

## Key Optimizations

1. **No Arc Wrapper**: `reqwest::Client` already uses Arc internally, so we clone it directly
2. **Branch-Free Rate Limiting**: RateLimiter always present with 0.0 = unlimited (no `Option<RateLimiter>`)
3. **Parallel Requests per VUser**: Each VUser can spawn N concurrent requests for higher throughput
4. **Request Template Caching**: `to_request_builder()` method pre-builds requests efficiently
5. **Bounded Channels**: Prevent memory explosion with 10k capacity limits
6. **Lock-Free Architecture**: Zero mutexes, zero contention, pure message passing

---

## Status

**Architecture Complete** - All traits defined, data flows documented, ready for runtime implementation.

**Next Steps:**
1. Implement IPOG planner algorithm
2. Complete Scheduler orchestration loop
3. Implement Executor VUser spawning logic
4. Build MetricsHub background tasks
5. Add WebSocket streaming server
6. Implement storage backends (SQLite, JSONL)

---
