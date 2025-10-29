use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ScenarioId(String);

/// Main test scenario configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// Unique identifier for this scenario
    id: ScenarioId,
    scenario: Scenario,
}

#[derive(Debug, Serialize, Deserialize)]
enum Scenario {
    LoadTesting(LoadTesting),
    Security(Security),
}

#[derive(Debug, Serialize, Deserialize)]
struct Security;

#[derive(Debug, Serialize, Deserialize)]
struct LoadTesting;
