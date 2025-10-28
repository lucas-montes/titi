# Elerem Fuzzer - Mermaid Architecture Diagrams

## Complete System Architecture

```mermaid
graph TB
    subgraph User["🌐 USER / WEBSITE"]
        UserInput[/"JSON/YAML Config<br/>base_url, auth, scenarios"/]
        WebSocket[/"📊 WebSocket<br/>Real-time Metrics"/]
    end

    subgraph Layer1["⚙️ LAYER 1: CONFIGURATION"]
        Config["Parse & Validate<br/>• JSON/YAML parsing<br/>• Parameter extraction<br/>• Auth validation"]
    end

    subgraph Layer2["🧠 LAYER 2: PLANNER"]
        Planner["TestPlanner: Iterator&lt;Item = Case&gt;"]
        IPOG["IPOG<br/>Combinatorial"]
        IPOGC["IPOG-C<br/>+Constraints"]
        Adaptive["T-wise Adaptive<br/>Dynamic Coverage"]
        Boundary["Boundary<br/>Edge Cases"]

        Planner -.-> IPOG
        Planner -.-> IPOGC
        Planner -.-> Adaptive
        Planner -.-> Boundary

        Case["Case {<br/>id, request_template,<br/>executor_type,<br/>assertions,<br/>stopping_conditions}"]
    end

    subgraph Layer3["🎯 LAYER 3: SCHEDULER"]
        Scheduler["Scheduler<br/>Orchestration Loop"]
        MetricsHub["MetricsHub"]
        MetricsStream["MetricsStream<br/>WebSocket Server<br/>60s circular buffer"]
        MetricsRecorder["MetricsRecorder<br/>DB/File/Memory<br/>Buffered writes"]

        MetricsHub --> MetricsStream
        MetricsHub --> MetricsRecorder
    end

    subgraph Layer4["⚡ LAYER 4: EXECUTOR"]
        Exec1["Executor 1<br/>Case: login-test<br/>10 VUsers"]
        Exec2["Executor 2<br/>Case: search-test<br/>50 VUsers"]
        Exec3["Executor N<br/>Case: checkout-test<br/>5 VUsers"]

        ExecLogic["Per Executor:<br/>• Spawn N VUsers<br/>• Aggregate metrics (1/sec)<br/>• Check assertions<br/>• Check stopping conditions<br/>• Calculate p50/p95/p99"]
    end

    subgraph Layer5["👥 LAYER 5: VIRTUAL USERS"]
        VU1["VUser 1<br/>Sequential"]
        VU2["VUser 2<br/>Parallel 3x"]
        VUN["VUser N<br/>Sequential"]

        VULogic["Per VUser:<br/>• Rate limit (0.0 = unlimited)<br/>• Build request<br/>• Execute HTTP<br/>• Collect metrics<br/>• Send upstream"]
    end

    API["🌍 TARGET API<br/>(Under Test)"]

    %% Flow connections
    UserInput --> Config
    Config --> Planner
    Planner -->|"Iterator::next()"| Case
    Case --> Scheduler
    Scheduler -->|"spawn tokio task"| Exec1
    Scheduler -->|"spawn tokio task"| Exec2
    Scheduler -->|"spawn tokio task"| Exec3

    Exec1 -.->|"illustrates"| ExecLogic

    Exec1 -->|"spawn tokio tasks"| VU1
    Exec1 -->|"spawn tokio tasks"| VU2
    Exec1 -->|"spawn tokio tasks"| VUN

    VU1 -.->|"illustrates"| VULogic

    VU1 -->|"HTTP requests"| API
    VU2 -->|"HTTP requests"| API
    VUN -->|"HTTP requests"| API

    %% Metrics flow (bottom-up)
    VU1 -->|"VUserMetrics<br/>mpsc channel"| Exec1
    VU2 -->|"VUserMetrics<br/>mpsc channel"| Exec1
    VUN -->|"VUserMetrics<br/>mpsc channel"| Exec1

    Exec1 -->|"ExecutorSnapshot<br/>mpsc channel"| Scheduler
    Exec2 -->|"ExecutorSnapshot<br/>mpsc channel"| Scheduler
    Exec3 -->|"ExecutorSnapshot<br/>mpsc channel"| Scheduler

    Scheduler --> MetricsHub
    MetricsStream -->|"GlobalSnapshot @ 2/sec"| WebSocket

    %% Feedback loop
    Scheduler -.->|"PlannerFeedback<br/>(adaptive)"| Planner

    style User fill:#e1f5ff
    style Layer1 fill:#fff4e6
    style Layer2 fill:#f3e5f5
    style Layer3 fill:#e8f5e9
    style Layer4 fill:#fff3e0
    style Layer5 fill:#fce4ec
    style API fill:#ffebee
```

---

## Data Flow & Sequence

```mermaid
sequenceDiagram
    participant U as User/Website
    participant C as Config
    participant P as Planner
    participant S as Scheduler
    participant E as Executor
    participant V as VirtualUser
    participant A as Target API
    participant M as MetricsHub

    U->>C: JSON/YAML config
    C->>P: Configuration

    loop Test Generation
        P->>P: Generate Case (lazy)
        P->>S: Iterator::next() → Case
        S->>E: spawn(case)

        par Spawn VUsers
            E->>V: spawn N VUsers
        end

        loop Execute Requests
            V->>V: Rate limit (0.0 = unlimited)
            V->>V: Build request via to_request_builder()
            V->>A: HTTP Request
            A-->>V: HTTP Response
            V->>E: VUserMetrics (mpsc)
        end

        loop Every 1 second
            E->>E: Aggregate VUserMetrics
            E->>E: Calculate p50, p95, p99
            E->>E: Check Assertions
            E->>E: Check StoppingConditions
            E->>S: ExecutorSnapshot (mpsc)
        end

        S->>M: Update GlobalSnapshot
        M->>U: Stream via WebSocket (2/sec)
        M->>M: Record to DB/File

        S-->>P: PlannerFeedback (adaptive)
    end
```

---

## Metrics Aggregation Hierarchy

```mermaid
graph TD
    subgraph Level1["📊 LEVEL 1: VUserMetrics (Individual Request)"]
        V1["VUser 1<br/>status: 200<br/>time: 45ms<br/>success: true"]
        V2["VUser 2<br/>status: 200<br/>time: 52ms<br/>success: true"]
        VN["VUser N<br/>status: 500<br/>time: 3s<br/>success: false"]
    end

    subgraph Level2["📈 LEVEL 2: ExecutorSnapshot (Per-Case)"]
        E1["Executor: login-test<br/>total: 1523<br/>successful: 1520<br/>failed: 3<br/>rps: 25.4<br/>p50: 45ms, p95: 87ms, p99: 152ms<br/>vusers: 10"]
        E2["Executor: search-test<br/>total: 3421<br/>successful: 3401<br/>failed: 20<br/>rps: 57.1<br/>p50: 32ms, p95: 65ms, p99: 120ms<br/>vusers: 50"]
    end

    subgraph Level3["🌍 LEVEL 3: GlobalSnapshot (All Cases)"]
        G["GlobalSnapshot<br/>total_requests: 5234<br/>total_successful: 5201<br/>total_failed: 33<br/>overall_rps: 87.2<br/>executors: [login-test, search-test, ...]"]
    end

    subgraph Output["📡 OUTPUT"]
        WS["WebSocket<br/>Broadcast @ 2/sec<br/>60s history"]
        DB["Database<br/>SQLite/JSONL<br/>Flush @ 100 snapshots"]
    end

    V1 -->|mpsc channel| E1
    V2 -->|mpsc channel| E1
    VN -->|mpsc channel| E1

    E1 -->|mpsc channel| G
    E2 -->|mpsc channel| G

    G --> WS
    G --> DB

    style Level1 fill:#fce4ec
    style Level2 fill:#fff3e0
    style Level3 fill:#e8f5e9
    style Output fill:#e1f5ff
```

---

## Concurrency Model

```mermaid
graph TB
    subgraph Scheduler["Scheduler Task (Single)"]
        S["• Poll planner.next()<br/>• Spawn executors<br/>• Coordinate MetricsHub<br/>• No locks"]
    end

    subgraph Executor1["Executor 1 Task"]
        E1["Case: login-test<br/>Manages 10 VUsers"]
        E1V1["VUser 1"]
        E1V2["VUser 2"]
        E1V10["VUser 10"]
    end

    subgraph Executor2["Executor 2 Task"]
        E2["Case: search-test<br/>Manages 50 VUsers"]
        E2V["VUser 1..50"]
    end

    subgraph MetricsHubTask["MetricsHub Tasks"]
        Stream["MetricsStream<br/>WebSocket Server<br/>Broadcast @ 2/sec"]
        Recorder["MetricsRecorder<br/>Background Flusher<br/>Write @ 100 snapshots"]
    end

    Client["reqwest::Client<br/>(Arc internally)<br/>Shared via clone()"]

    S --> E1
    S --> E2
    S --> MetricsHubTask

    E1 --> E1V1
    E1 --> E1V2
    E1 --> E1V10

    E2 --> E2V

    E1V1 -.->|"clone()"| Client
    E1V2 -.->|"clone()"| Client
    E1V10 -.->|"clone()"| Client
    E2V -.->|"clone()"| Client

    MetricsHubTask --> Stream
    MetricsHubTask --> Recorder

    style Scheduler fill:#e8f5e9
    style Executor1 fill:#fff3e0
    style Executor2 fill:#fff3e0
    style MetricsHubTask fill:#e1f5ff
    style Client fill:#f3e5f5
```

---

## Validation Flow

```mermaid
graph LR
    subgraph Case["Case (Declarative)"]
        A["Assertions<br/>• StatusCode(200)<br/>• P95ResponseTime(100ms)<br/>• SuccessRate(99%)"]
        SC["StoppingConditions<br/>• FailureRate(15%)<br/>• ConsecutiveErrors(50)<br/>• P95ResponseTime(5s)"]
    end

    subgraph Executor["Executor (Runtime)"]
        Check["Every 1 second:<br/>1. Check assertions<br/>2. Check stopping conditions"]
    end

    subgraph Result["Result"]
        Pass["✅ All Pass<br/>Continue"]
        Fail["❌ Assertion Failed<br/>Test FAILED"]
        Stop["⏹️ Stopping Condition<br/>Graceful Stop"]
    end

    Case --> Executor
    Check --> Pass
    Check --> Fail
    Check --> Stop

    style Case fill:#f3e5f5
    style Executor fill:#fff3e0
    style Pass fill:#e8f5e9
    style Fail fill:#ffebee
    style Stop fill:#fff4e6
```

---

## Feedback Loop (Adaptive Planning)

```mermaid
sequenceDiagram
    participant P as Planner
    participant S as Scheduler
    participant E as Executor

    Note over P: Initial state:<br/>Generate 2-way combos

    P->>S: next() → Case
    S->>E: spawn(case)

    E->>E: Execute with N VUsers
    E->>E: Collect metrics

    alt Assertion Failed
        E-->>S: Err(assertion failed)
        S-->>P: feedback(failure_rate: 0.15)
        Note over P: Adjust: Generate<br/>3-way combos in<br/>problem area
    else Stopping Condition
        E-->>S: Ok(stopped early)
        S-->>P: feedback(failure_rate: 0.02)
        Note over P: Adjust: Reduce<br/>coverage in<br/>successful area
    else Success
        E-->>S: ExecutorSnapshot(success)
        S-->>P: feedback(success: true)
        Note over P: Continue<br/>current strategy
    end

    P->>S: next() → Adjusted Case
```

---

## Component Communication

```mermaid
graph LR
    subgraph Channels["Message Passing (No Locks)"]
        C1["mpsc::Sender&lt;VUserMetrics&gt;<br/>Capacity: 10,000<br/>Bounded"]
        C2["mpsc::Sender&lt;ExecutorSnapshot&gt;<br/>Capacity: 10,000<br/>Bounded"]
    end

    subgraph Shared["Shared State (Thread-Safe)"]
        Client["reqwest::Client<br/>Arc&lt;HttpPool&gt; internally<br/>Connection pooling"]
    end

    subgraph NoSharing["Share-Nothing"]
        VUser["VirtualUser<br/>• Own rate_limiter<br/>• Own metrics<br/>• No locks"]
    end

    VUser -->|"try_send()"| C1
    C1 --> Executor
    Executor -->|"try_send()"| C2
    C2 --> Scheduler

    VUser -.->|"clone()"| Client

    style Channels fill:#e8f5e9
    style Shared fill:#e1f5ff
    style NoSharing fill:#fff4e6
```

---

## Test Case Structure

```mermaid
classDiagram
    class Case {
        +String id
        +String name
        +RequestTemplate request_template
        +ExecutorType executor_type
        +Vec~Assertion~ assertions
        +Vec~StoppingCondition~ stopping_conditions
    }

    class RequestTemplate {
        +Method method
        +String url
        +Vec~Header~ headers
        +Option~Vec~u8~~ body
        +Vec~QueryParam~ query_params
        +to_request_builder(client) RequestBuilder
    }

    class ExecutorType {
        <<enumeration>>
        SingleShot
        ConstantVus
        RampingVus
        ConstantArrivalRate
    }

    class Assertion {
        <<enumeration>>
        StatusCode(u16)
        P95ResponseTime(Duration)
        SuccessRate(f64)
        ErrorRate(f64)
    }

    class StoppingCondition {
        <<enumeration>>
        FailureRate(f64)
        ConsecutiveErrors(usize)
        P95ResponseTime(Duration)
        TotalRequests(usize)
    }

    Case --> RequestTemplate
    Case --> ExecutorType
    Case --> Assertion
    Case --> StoppingCondition
```

---

## Key Optimizations

```mermaid
mindmap
  root((Optimizations))
    No Arc Wrapper
      reqwest::Client has Arc internally
      Just clone() it directly
    Branch-Free Rate Limiting
      RateLimiter always present
      0.0 = unlimited
      No Option branching
    Parallel Requests
      Each VUser spawns N concurrent
      Higher throughput
      Configurable per VUser
    Request Template Caching
      to_request_builder() method
      Pre-build once
      Reuse many times
    Bounded Channels
      10k capacity limit
      Prevent memory explosion
      Non-blocking try_send()
    Lock-Free Architecture
      Zero mutexes
      Zero RwLocks
      Pure message passing
      Share-nothing VUsers
```

---

## Performance Targets

```mermaid
graph LR
    subgraph Targets["Performance Goals"]
        T1["Throughput<br/>10k+ RPS<br/>per machine"]
        T2["Latency Overhead<br/>&lt;1ms<br/>per request"]
        T3["Memory<br/>&lt;100MB base<br/>+1KB per VUser"]
        T4["Scalability<br/>Linear with<br/>CPU cores"]
    end

    subgraph Strategies["Strategies"]
        S1["Async I/O<br/>tokio runtime"]
        S2["Lock-free<br/>design"]
        S3["Lazy generation<br/>bounded buffers"]
        S4["Share-nothing<br/>VUser arch"]
    end

    T1 --> S1
    T1 --> S2
    T2 --> S2
    T3 --> S3
    T4 --> S4

    style Targets fill:#e8f5e9
    style Strategies fill:#e1f5ff
```

---

These Mermaid diagrams provide interactive, visual representations of the Elerem Fuzzer architecture that can be rendered in GitHub, documentation sites, or any Markdown viewer that supports Mermaid syntax.
