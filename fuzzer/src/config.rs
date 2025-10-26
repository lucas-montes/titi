use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ScenarioId(String);

/// Main test scenario configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct TestScenario {
    /// Unique identifier for this scenario
    id: ScenarioId,
}
