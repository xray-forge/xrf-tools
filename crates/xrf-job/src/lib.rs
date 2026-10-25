#![doc = include_str!("../README.md")]

mod emission_gate;
mod execution_origin;
mod execution_plan;
mod execution_request;
mod job_handle;
mod job_outcome;
mod job_progress;
mod job_scope;
#[cfg(feature = "rayon")]
mod pool_once_lock;
mod progress_level;
mod progress_sink;
mod progress_unit;

#[cfg(test)]
mod tests;

pub use crate::execution_origin::ExecutionOrigin;
pub use crate::execution_plan::ExecutionPlan;
pub use crate::execution_request::ExecutionRequest;
pub use crate::job_handle::{DEFAULT_PROGRESS_INTERVAL, JobHandle};
pub use crate::job_outcome::JobOutcome;
pub use crate::job_progress::JobProgress;
pub use crate::job_scope::JobScope;
#[cfg(feature = "rayon")]
pub use crate::pool_once_lock::PoolOnceLock;
pub use crate::progress_level::ProgressLevel;
pub use crate::progress_sink::{NoopSink, ProgressSink, RecordingSink};
pub use crate::progress_unit::ProgressUnit;
