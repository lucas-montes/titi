mod config;
mod client;
mod metrics;
mod planner;
mod scheduler;

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
