use crate::planner::Plan;

pub struct Scheduler<P: Plan> {
    planner: P,
    metrics_stream: MetricsStream,
}
