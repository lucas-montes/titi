use crate::config::Configuration;

pub trait Plan: Iterator<Item = Self::Case> + From<Configuration> {
    type Case;

    fn generate(&mut self);
}

pub struct Planner {}

impl From<Configuration> for Planner {
    fn from(_config: Configuration) -> Self {
        // Implementation to create a TestPlanner from Configuration
        Self {}
    }
}

impl Iterator for Planner {
    type Item = Case;

    fn next(&mut self) -> Option<Self::Item> {
        // Implementation to generate the next test case
        None
    }
}

struct Case;
