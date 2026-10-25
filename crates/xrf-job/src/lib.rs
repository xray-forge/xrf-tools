#![doc = include_str!("../README.md")]

mod emission_gate;
mod execution_origin;
mod execution_plan;
mod execution_request;
mod job_handle;
mod job_outcome;
mod job_progress;
mod job_scope;
mod job_state;
mod level_state;
mod noop_sink;
#[cfg(feature = "rayon")]
mod pool_once_lock;
mod progress_level;
mod progress_sink;
mod progress_unit;
mod recording_sink;

#[cfg(test)]
mod tests;

pub use crate::execution_origin::ExecutionOrigin;
pub use crate::execution_plan::ExecutionPlan;
pub use crate::execution_request::ExecutionRequest;
pub use crate::job_handle::{DEFAULT_PROGRESS_INTERVAL, JobHandle};
pub use crate::job_outcome::JobOutcome;
pub use crate::job_progress::JobProgress;
pub use crate::job_scope::JobScope;
pub use crate::noop_sink::NoopSink;
#[cfg(feature = "rayon")]
pub use crate::pool_once_lock::PoolOnceLock;
pub use crate::progress_level::ProgressLevel;
pub use crate::progress_sink::ProgressSink;
pub use crate::progress_unit::ProgressUnit;
pub use crate::recording_sink::RecordingSink;
