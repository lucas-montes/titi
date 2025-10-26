# Complete Flow Diagram

## High-Level Flow

This document shows the complete flow of data and control through the fuzzer, including feedback loops.

### Complete System Flow

```
                                    ┌──────────────────┐
                                    │  User/Website    │
                                    └────────┬─────────┘
                                             │
                                             ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                              1. CONFIGURATION                              │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  Config { base_url, auth, scenarios, ... }                          │  │
│  │  Format: JSON, YAML, logs, manual entry                             │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────┬───────────────────────────────────────┘
                                     │
                                     ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                               2. PLANNER                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  impl Iterator<Item = Case>                                         │  │
│  │                                                                      │  │
│  │  Algorithms:                                                         │  │
│  │  • IPOG: Combinatorial testing                                      │  │
│  │  • IPOG-C: With constraint handling                                 │  │
│  │  • T-wise Adaptive: Dynamic coverage adjustment                     │  │
│  │  • Boundary Value Analysis: Edge cases                              │  │
│  │                                                                      │  │
│  │  fn next() -> Option<Case>  ◄─────┐                                 │  │
│  │  fn feedback(PlannerFeedback)     │ Adaptive feedback loop          │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────┬───────────────────────────────────────┘
                                     │                                     ▲
                                     │ Iterator<Item = Case>               │
                                     ▼                                     │
┌────────────────────────────────────────────────────────────────────────────┐
│                              3. SCHEDULER                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  Main orchestration loop:                                           │  │
│  │                                                                      │  │
│  │  1. Poll planner.next() for new cases                               │  │
│  │  2. Spawn Executor for each case                                    │  │
│  │  3. Receive ExecutorSnapshots via channel                           │  │
│  │  4. Update MetricsHub (stream + record)                             │  │
│  │  5. Check global stopping conditions                                │  │
│  │  6. Send feedback to planner if adaptive ────────────────────────┐  │  │
│  │                                                                   │  │  │
│  │  MetricsHub {                                                     │  │  │
│  │    stream: MetricsStream,      ──┐                                │  │  │
│  │    recorder: MetricsRecorder     │                                │  │  │
│  │  }                               │                                │  │  │
│  └──────────────────────────────────┼────────────────────────────────┼──┘  │
└───────────────────────┬─────────────┼────────────────────────────────┼─────┘
                        │             │                                │
                        │             │ WebSocket stream               │
                        │             ▼                                │
                        │      ┌─────────────────┐                    │
                        │      │   Website       │                    │
                        │      │   (real-time)   │                    │
                        │      └─────────────────┘                    │
                        │                                              │
                        │ spawn Executor                  PlannerFeedback
                        │                                              │
                        ▼                                              │
┌─────────────────────────────────────────────────────────────────────┼──────┐
│                              4. EXECUTOR                             │      │
│  ┌──────────────────────────────────────────────────────────────────┼───┐  │
│  │  Manages single test case:                                       │   │  │
│  │                                                                   │   │  │
│  │  1. Spawn N VirtualUsers based on ExecutorType                   │   │  │
│  │  2. Collect VUserMetrics via channel                             │   │  │
│  │  3. Aggregate metrics (p50, p95, p99, RPS)                       │   │  │
│  │  4. Check case assertions every second                           │   │  │
│  │  5. Check stopping conditions                                    │   │  │
│  │  6. Send ExecutorSnapshot to Scheduler ─────────────────────────┼───┘  │
│  │                                                                   │      │
│  │  MetricsCollector {                                               │      │
│  │    aggregated: AggregatedMetrics,                                │      │
│  │    response_times: VecDeque<Duration>  # for percentiles         │      │
│  │  }                                                                │      │
│  └───────────────────────────────────────────────────────────────────────┘  │
└───────────────────────┬─────────────────────────────────────────────────────┘
                        │
                        │ spawn VirtualUser (x N)
                        │
                        ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                           5. VIRTUAL USERS                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  VirtualUser { id, http_client: Arc<HttpClient>, ... }             │  │
│  │                                                                      │  │
│  │  Loop:                                                               │  │
│  │    1. Apply rate limiter if configured                              │  │
│  │    2. Build HTTP request from RequestTemplate                       │  │
│  │    3. Execute request                ──────────────┐                │  │
│  │    4. Collect metrics (duration, status, error)    │ HTTP           │  │
│  │    5. Send VUserMetrics upstream via channel       ▼                │  │
│  │                                              ┌──────────────┐        │  │
│  │  Share-nothing architecture                  │  Target API  │        │  │
│  │  Shared HttpClient (Arc) for pooling         └──────────────┘        │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────┘
```

## Metrics Flow Detail

```
VirtualUser                    Executor                    Scheduler/MetricsHub
    │                             │                              │
    │ VUserMetrics                │                              │
    ├────────────────────────────►│                              │
    │ { vuser_id, timestamp,      │                              │
    │   status_code, time,        │                              │
    │   success, error }          │                              │
    │                             │                              │
    │                             │ Aggregate:                   │
    │                             │ • Calculate p50/p95/p99      │
    │                             │ • Count success/failed       │
    │                             │ • Calculate RPS              │
    │                             │ • Track status codes         │
    │                             │                              │
    │                             │ ExecutorSnapshot             │
    │                             ├─────────────────────────────►│
    │                             │ { case_id, timestamp,        │
    │                             │   total_requests,            │
    │                             │   successful, failed,        │
    │                             │   current_rps, p95, ... }    │
    │                             │                              │
    │                             │                              │ GlobalSnapshot
    │                             │                              ├──────────────►WebSocket
    │                             │                              │ Aggregate all
    │                             │                              │ executors
    │                             │                              │
    │                             │                              │
    │                             │                              ├──────────────►Storage
    │                             │                              │ Buffered
    │                             │                              │ writes
```

## Validation Flow

```
┌──────────────┐
│     Case     │  Defines declarative rules
└──────┬───────┘
       │ Contains:
       │ • assertions: Vec<Assertion>
       │ • stopping_conditions: Vec<StoppingCondition>
       │
       ▼
┌──────────────┐
│   Executor   │  Enforces rules at runtime
└──────┬───────┘
       │ Every second:
       │ 1. Check all assertions
       │ 2. If any fail → return Err(...)
       │ 3. Check stopping conditions
       │ 4. If any match → return Ok(()) (early stop)
       │
       ▼
┌──────────────┐
│  Scheduler   │  Global coordination
└──────────────┘
       │ When Executor returns:
       │ • Success: Continue with next case
       │ • Failure: Log, optionally stop all
       │ • Check global stopping conditions
       │   (e.g., total failure rate across all cases)
```

## Feedback Loop (Adaptive Planning)

```
                    ┌───────────────────────────────────────┐
                    │         Adaptive Planner              │
                    │  (T-wise Adaptive, ML-based, etc.)    │
                    └───────────┬───────────────────────────┘
                                │                           ▲
                                │ next()                    │
                                │ Returns: Case             │ feedback()
                                ▼                           │
                    ┌───────────────────────────────────────┤
                    │          Scheduler                    │
                    └───────────┬───────────────────────────┘
                                │                           ▲
                                │ spawn                     │
                                ▼                           │
                    ┌───────────────────────────────────────┤
                    │          Executor                     │
                    │  Executes case, collects metrics      │
                    └───────────────────────────────────────┘
                                │
                                │ ExecutorSnapshot
                                │ { success, failure_rate,
                                │   response_time_p95, ... }
                                │
                                └──────────► Scheduler analyzes results
                                            Sends PlannerFeedback:
                                            • Which case was executed
                                            • Success/failure
                                            • Performance metrics

                                            Planner adjusts:
                                            • Increase coverage in failing areas
                                            • Skip high-performing areas
                                            • Adjust parameter combinations
```

## Lifecycle

### Startup

```
1. Load Configuration
      ↓
2. Create Planner (IPOG, Adaptive, etc.)
      ↓
3. Create Scheduler with MetricsHub
      ↓
4. Start MetricsStream task (WebSocket server)
      ↓
5. Start MetricsRecorder task (periodic flush to storage)
      ↓
6. Enter main loop
```

### Main Loop

```
loop {
    1. case = planner.next()
    2. if case.is_none() → break
    3. executor = Executor::new(case, http_client, metrics_tx)
    4. spawn executor.run()
    5. receive ExecutorSnapshot
    6. update MetricsHub
    7. check global stopping conditions
    8. if adaptive → planner.feedback(results)
}
```

### Shutdown

```
1. Stop polling planner
      ↓
2. Wait for all executors to finish
      ↓
3. Flush metrics recorder buffer
      ↓
4. Close WebSocket connections
      ↓
5. Generate final report
```

## Concurrency Model

```
┌─────────────────────────────────────────────────────────────┐
│                        Scheduler                            │
│  Single-threaded orchestrator (tokio task)                  │
│  • Polls planner sequentially                               │
│  • Spawns executors (tokio::spawn)                          │
│  • Receives metrics via mpsc channel                        │
└──────────────┬──────────────────────────────────────────────┘
               │
               │ Spawns multiple executors (parallel or sequential)
               │
    ┌──────────┴──────────┬──────────────┬──────────────┐
    ▼                     ▼              ▼              ▼
┌─────────┐          ┌─────────┐    ┌─────────┐    ┌─────────┐
│Executor │          │Executor │    │Executor │    │Executor │
│  (1)    │          │  (2)    │    │  (3)    │    │  (N)    │
└────┬────┘          └────┬────┘    └────┬────┘    └────┬────┘
     │                    │              │              │
     │ Spawns VUsers      │              │              │
     │                    │              │              │
 ┌───┴───┬───┬───┐    ┌──┴──┬──┬──┐   ...           ...
 ▼       ▼   ▼   ▼    ▼     ▼  ▼  ▼
VU1     VU2 VU3 VU4  VU1   VU2 ...
```

- **Scheduler**: Single tokio task, orchestrates everything
- **Executors**: Multiple concurrent tasks (1 per case)
- **VUsers**: Multiple concurrent tasks (N per executor)
- **Communication**: mpsc channels (non-blocking, bounded)

## Error Handling

```
VirtualUser
    ├─ Request Error → VUserMetrics { success: false, error: Some(...) }
    └─ Channel Error → Stop execution

Executor
    ├─ Assertion Failed → return Err("Assertion failed")
    ├─ Stopping Condition → return Ok(())
    └─ All VUsers died → return Err("No active VUsers")

Scheduler
    ├─ Executor Error → Log, optionally stop all
    ├─ Global Condition → Stop polling planner
    └─ MetricsHub Error → Log, continue execution
```

## Performance Optimizations

1. **Lazy Generation**: Planner uses Iterator pattern - no upfront memory cost
2. **Share-Nothing VUsers**: True parallelism, no locks
3. **Shared Client**: Connection pooling via Arc<HttpClient>
4. **Bounded Channels**: Prevent memory explosion under load
5. **Buffered Recording**: Batch writes to storage
6. **Lock-Free Metrics**: VUsers send via channels, no shared state
7. **Streaming History**: Fixed-size circular buffer (60 seconds)
