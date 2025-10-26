pub mod config;
pub mod planner;
pub mod scheduler;
pub mod executor;
pub mod vuser;

// Fuzzer Architecture (5 Layers)
//
// 1. Configuration: User input in any format (JSON, YAML, logs, manual)
//    - High-level representation of testing goals
//    - Authentication, base URLs, test scenarios
//
// 2. Planner: Generates test cases using algorithms
//    - IPOG-C: T-wise testing with constraint handling
//    - T-wise Adaptive: Dynamic coverage adjustment
//    - Boundary analysis, edge cases, assertions
//    - Iterator-based lazy generation
//    - Can receive feedback from Scheduler for adaptive planning
//
// 3. Scheduler: Orchestrates test execution
//    - Polls cases from Planner
//    - Manages parallel vs sequential execution
//    - Owns MetricsHub (Stream + Recorder)
//    - Provides feedback to adaptive planners
//    - Communicates with website via WebSocket
//
// 4. Executor: Manages a single test case
//    - Spawns and manages N virtual users
//    - Enforces case assertions (p95, success rate)
//    - Checks stopping conditions
//    - Collects and aggregates VUser metrics
//    - Sends snapshots to Scheduler
//
// 5. Virtual Users: Dumb workers executing requests
//    - Share-nothing architecture
//    - Shares HttpClient via Arc (connection pooling)
//    - Per-VUser or shared rate limiting
//    - Sends metrics upstream to Executor
