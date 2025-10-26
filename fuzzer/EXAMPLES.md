# Usage Examples

This document provides practical examples of how to use the Elerem Fuzzer.

## Basic Load Test

Simple load test with constant VUsers:

```rust
use fuzzer::{
    planner::{Case, RequestTemplate, Method, ExecutorType, Assertion},
    scheduler::{Scheduler, SchedulerConfig, StorageConfig},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple planner with one case
    let cases = vec![
        Case {
            id: "load-test-1".into(),
            name: "Basic Load Test".into(),
            request_template: RequestTemplate {
                method: Method::Get,
                url: "https://api.example.com/health".into(),
                headers: vec![],
                body: None,
                query_params: vec![],
            },
            executor_type: ExecutorType::ConstantVus {
                vus: 10,
                duration: Duration::from_secs(60),
            },
            assertions: vec![
                Assertion::StatusCode(200),
                Assertion::P95ResponseTime(Duration::from_millis(100)),
                Assertion::SuccessRate(0.99),
            ],
            stopping_conditions: vec![],
        },
    ];

    let planner = ManualPlanner::new(cases);

    // Create scheduler
    let scheduler = Scheduler::new(planner, SchedulerConfig {
        enable_streaming: true,
        enable_recording: true,
        storage: StorageConfig::File("results.jsonl".into()),
        parallel_execution: false,
    });

    // Run tests
    scheduler.run().await?;

    Ok(())
}
```

## Ramping Load Test

Gradually increase load to find breaking point:

```rust
use fuzzer::planner::{ExecutorType, Stage, StoppingCondition};

let case = Case {
    id: "ramp-test".into(),
    name: "Ramping Load Test".into(),
    request_template: RequestTemplate {
        method: Method::Post,
        url: "https://api.example.com/users".into(),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
        ],
        body: Some(r#"{"name":"test"}"#.as_bytes().to_vec()),
        query_params: vec![],
    },
    executor_type: ExecutorType::RampingVus {
        stages: vec![
            Stage { duration: Duration::from_secs(10), target_vus: 10 },
            Stage { duration: Duration::from_secs(20), target_vus: 50 },
            Stage { duration: Duration::from_secs(30), target_vus: 100 },
            Stage { duration: Duration::from_secs(10), target_vus: 0 },
        ],
    },
    assertions: vec![
        Assertion::P95ResponseTime(Duration::from_millis(500)),
    ],
    stopping_conditions: vec![
        StoppingCondition::FailureRate(0.1),  // Stop if >10% failures
        StoppingCondition::P95ResponseTime(Duration::from_secs(5)),
    ],
};
```

## Stress Test with Error Tolerance

Push system to limits with early stopping:

```rust
let case = Case {
    id: "stress-test".into(),
    name: "Stress Test".into(),
    request_template: RequestTemplate {
        method: Method::Get,
        url: "https://api.example.com/expensive-operation".into(),
        headers: vec![],
        body: None,
        query_params: vec![],
    },
    executor_type: ExecutorType::ConstantArrivalRate {
        rate: 1000.0,  // 1000 requests per second
        duration: Duration::from_secs(300),
        max_vus: 500,
    },
    assertions: vec![
        Assertion::ErrorRate(0.05),  // Allow up to 5% errors
    ],
    stopping_conditions: vec![
        StoppingCondition::ConsecutiveErrors(100),
        StoppingCondition::P99ResponseTime(Duration::from_secs(10)),
        StoppingCondition::TotalRequests(100_000),
    ],
};
```

## API Testing with Multiple Endpoints

Test multiple endpoints in sequence:

```rust
let cases = vec![
    // Login
    Case {
        id: "login".into(),
        name: "User Login".into(),
        request_template: RequestTemplate {
            method: Method::Post,
            url: "https://api.example.com/login".into(),
            headers: vec![
                ("Content-Type".into(), "application/json".into()),
            ],
            body: Some(r#"{"email":"test@example.com","password":"test123"}"#.as_bytes().to_vec()),
            query_params: vec![],
        },
        executor_type: ExecutorType::SingleShot,
        assertions: vec![
            Assertion::StatusCode(200),
        ],
        stopping_conditions: vec![],
    },

    // Fetch data
    Case {
        id: "fetch-data".into(),
        name: "Fetch User Data".into(),
        request_template: RequestTemplate {
            method: Method::Get,
            url: "https://api.example.com/users/me".into(),
            headers: vec![
                ("Authorization".into(), "Bearer TOKEN".into()),
            ],
            body: None,
            query_params: vec![],
        },
        executor_type: ExecutorType::ConstantVus {
            vus: 5,
            duration: Duration::from_secs(10),
        },
        assertions: vec![
            Assertion::StatusCode(200),
            Assertion::P95ResponseTime(Duration::from_millis(50)),
        ],
        stopping_conditions: vec![],
    },

    // Update data
    Case {
        id: "update-data".into(),
        name: "Update User Profile".into(),
        request_template: RequestTemplate {
            method: Method::Put,
            url: "https://api.example.com/users/me".into(),
            headers: vec![
                ("Authorization".into(), "Bearer TOKEN".into()),
                ("Content-Type".into(), "application/json".into()),
            ],
            body: Some(r#"{"name":"New Name"}"#.as_bytes().to_vec()),
            query_params: vec![],
        },
        executor_type: ExecutorType::SingleShot,
        assertions: vec![
            Assertion::StatusCode(200),
        ],
        stopping_conditions: vec![],
    },
];

let planner = ManualPlanner::new(cases);
```

## Custom Planner Implementation

Create a custom planner with IPOG algorithm:

```rust
use fuzzer::planner::{TestPlanner, Case, PlannerFeedback, Configuration};

pub struct IpogPlanner {
    parameters: Vec<Parameter>,
    combinations: Vec<Combination>,
    current: usize,
    config: Configuration,
}

pub struct Parameter {
    name: String,
    values: Vec<String>,
}

struct Combination {
    values: Vec<usize>,
}

impl TestPlanner for IpogPlanner {
    fn from_config(config: Configuration) -> Self {
        // Extract parameters from config
        let parameters = extract_parameters(&config);

        // Generate t-way combinations
        let combinations = generate_ipog_combinations(&parameters, 2);

        Self {
            parameters,
            combinations,
            current: 0,
            config,
        }
    }

    fn algorithm_name(&self) -> &'static str {
        "IPOG"
    }
}

impl Iterator for IpogPlanner {
    type Item = Case;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.combinations.len() {
            return None;
        }

        let combo = &self.combinations[self.current];
        self.current += 1;

        // Build case from combination
        Some(build_case_from_combination(&self.config, &self.parameters, combo))
    }
}
```

## Adaptive Planner with Feedback

Implement adaptive planner that adjusts based on results:

```rust
pub struct AdaptivePlanner {
    base_cases: Vec<Case>,
    success_map: HashMap<String, f64>,
    config: Configuration,
}

impl TestPlanner for AdaptivePlanner {
    fn from_config(config: Configuration) -> Self {
        Self {
            base_cases: generate_base_cases(&config),
            success_map: HashMap::new(),
            config,
        }
    }

    fn algorithm_name(&self) -> &'static str {
        "Adaptive"
    }

    fn feedback(&mut self, feedback: PlannerFeedback) {
        // Update success map
        self.success_map.insert(
            feedback.case_id.clone(),
            if feedback.success { 1.0 } else { 0.0 }
        );

        // Analyze patterns
        if feedback.failure_rate > 0.5 {
            // Generate more cases similar to failing case
            let similar_cases = generate_similar_cases(&feedback.case_id, &self.config);
            self.base_cases.extend(similar_cases);
        }
    }
}

impl Iterator for AdaptivePlanner {
    type Item = Case;

    fn next(&mut self) -> Option<Self::Item> {
        // Return next case, prioritizing unexplored areas
        self.base_cases.pop()
    }
}
```

## Metrics Streaming to Website

WebSocket server for real-time metrics:

```rust
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

pub async fn start_metrics_stream(
    mut snapshot_rx: mpsc::Receiver<GlobalSnapshot>
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let mut local_rx = snapshot_rx.clone();

        tokio::spawn(async move {
            while let Some(snapshot) = local_rx.recv().await {
                // Serialize snapshot
                let json = serde_json::to_string(&snapshot).unwrap();

                // Send to WebSocket client
                if ws_sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        });
    }

    Ok(())
}
```

## Metrics Recording to Database

Persist metrics to SQLite:

```rust
use sqlx::{SqlitePool, Row};

pub struct DatabaseRecorder {
    pool: SqlitePool,
    buffer: Vec<ExecutorSnapshot>,
}

impl DatabaseRecorder {
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(db_path).await?;

        // Create tables
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY,
                case_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                total_requests INTEGER NOT NULL,
                successful INTEGER NOT NULL,
                failed INTEGER NOT NULL,
                current_rps REAL NOT NULL,
                p50_ms INTEGER NOT NULL,
                p95_ms INTEGER NOT NULL,
                p99_ms INTEGER NOT NULL
            )
        "#)
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            buffer: Vec::new(),
        })
    }

    pub async fn record(&mut self, snapshot: ExecutorSnapshot) -> Result<(), sqlx::Error> {
        self.buffer.push(snapshot);

        // Flush every 100 snapshots
        if self.buffer.len() >= 100 {
            self.flush().await?;
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), sqlx::Error> {
        for snapshot in self.buffer.drain(..) {
            sqlx::query(r#"
                INSERT INTO metrics (
                    case_id, timestamp, total_requests, successful, failed,
                    current_rps, p50_ms, p95_ms, p99_ms
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(&snapshot.case_id)
            .bind(snapshot.timestamp.elapsed().as_secs() as i64)
            .bind(snapshot.total_requests as i64)
            .bind(snapshot.successful as i64)
            .bind(snapshot.failed as i64)
            .bind(snapshot.current_rps)
            .bind(snapshot.p50.as_millis() as i64)
            .bind(snapshot.p95.as_millis() as i64)
            .bind(snapshot.p99.as_millis() as i64)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
```

## Configuration Examples

### JSON Configuration

```json
{
  "base_url": "https://api.example.com",
  "auth": {
    "type": "bearer",
    "token": "YOUR_TOKEN"
  },
  "scenarios": [
    {
      "name": "Load Test",
      "cases": [
        {
          "id": "case-1",
          "endpoint": "/users",
          "method": "GET",
          "executor": {
            "type": "constant_vus",
            "vus": 10,
            "duration": "60s"
          },
          "assertions": [
            {"type": "status_code", "value": 200},
            {"type": "p95_response_time", "value": "100ms"},
            {"type": "success_rate", "value": 0.99}
          ]
        }
      ]
    }
  ]
}
```

### YAML Configuration

```yaml
base_url: https://api.example.com
auth:
  type: bearer
  token: YOUR_TOKEN

scenarios:
  - name: Load Test
    cases:
      - id: case-1
        endpoint: /users
        method: GET
        executor:
          type: constant_vus
          vus: 10
          duration: 60s
        assertions:
          - type: status_code
            value: 200
          - type: p95_response_time
            value: 100ms
          - type: success_rate
            value: 0.99
```

## Running Tests

### Command Line

```bash
# Run with default config
cargo run --release

# Run with custom config
cargo run --release -- --config test-config.json

# Run with specific planner
cargo run --release -- --planner ipog --config test-config.json

# Run with metrics streaming enabled
cargo run --release -- --stream --port 8080

# Run with recording to database
cargo run --release -- --record --db results.sqlite
```

### Programmatic

```rust
use fuzzer::{Config, Scheduler, IpogPlanner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = Config::from_file("test-config.json").await?;

    // Create planner
    let planner = IpogPlanner::from_config(config);

    // Create scheduler
    let scheduler = Scheduler::new(planner, SchedulerConfig::default());

    // Run
    let results = scheduler.run().await?;

    // Print summary
    println!("Total requests: {}", results.total_requests);
    println!("Success rate: {:.2}%", results.success_rate * 100.0);
    println!("P95 response time: {:?}", results.p95);

    Ok(())
}
```

## Integration with Website

### Real-time Dashboard

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8080');

ws.onmessage = (event) => {
  const snapshot = JSON.parse(event.data);

  // Update dashboard
  updateChart(snapshot.overall_rps);
  updateMetrics({
    requests: snapshot.total_requests,
    success: snapshot.total_successful,
    failed: snapshot.total_failed,
  });

  // Update per-case metrics
  snapshot.executors.forEach(executor => {
    updateCaseCard(executor.case_id, {
      rps: executor.current_rps,
      p95: executor.p95,
      success_rate: executor.successful / executor.total_requests,
    });
  });
};
```

### Control API

```rust
use axum::{Router, routing::{get, post}, Json};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/tests/start", post(start_test))
        .route("/api/tests/stop", post(stop_test))
        .route("/api/tests/status", get(get_status))
        .route("/api/tests/results", get(get_results));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn start_test(Json(config): Json<Config>) -> Json<TestId> {
    // Start test in background
    let test_id = spawn_test(config).await;
    Json(test_id)
}

async fn stop_test(Json(test_id): Json<TestId>) -> Json<Status> {
    // Stop running test
    stop_test_by_id(test_id).await;
    Json(Status::Stopped)
}
```

## Conclusion

These examples show how to:
1. Create basic and advanced test cases
2. Implement custom planners
3. Stream metrics to WebSocket clients
4. Record metrics to persistent storage
5. Integrate with web dashboards
6. Control tests via API

The architecture is flexible and supports many use cases from simple load tests to complex adaptive testing scenarios.
