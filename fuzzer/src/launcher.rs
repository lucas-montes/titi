use crate::config::Configuration;
use crate::planner::TestPlanner;

use tokio::sync::mpsc;
use tokio::task::JoinSet;

/// FuzzerLauncher is the main entry point for starting fuzzer tests.
///
/// It receives a configuration and launches the appropriate planner,
/// then orchestrates the entire test execution through the scheduler.
pub struct Launcher {
    re: mpsc::Receiver<Configuration>,
    tasks: JoinSet<()>,
}

impl Launcher {
    /// Create a new fuzzer launcher with the given configuration and planner.
    pub fn new(config: Configuration) -> Self
    {
        let planner = P::from_config(config.clone());

        // Create default scheduler config based on configuration
        let scheduler_config = Self::create_scheduler_config(&config);

        Self {
            config,
            planner,
            scheduler_config,
        }
    }

}
