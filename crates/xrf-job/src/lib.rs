#![doc = include_str!("../README.md")]

mod emission_gate;
mod execution_origin;
mod execution_plan;
mod execution_request;
mod job_handle;
mod job_outcome;
mod job_progress;
mod job_scope;
mod logging_sink;
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
pub use crate::logging_sink::LoggingSink;
pub use crate::progress_level::ProgressLevel;
pub use crate::progress_sink::{NoopSink, ProgressSink, RecordingSink};
pub use crate::progress_unit::ProgressUnit;
