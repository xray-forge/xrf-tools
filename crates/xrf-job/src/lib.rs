#![doc = include_str!("../README.md")]

mod job_handle;
mod job_outcome;
mod job_progress;
mod job_scope;
mod progress_level;
mod progress_sink;
mod progress_unit;

#[cfg(test)]
mod tests;

pub use crate::job_handle::{DEFAULT_PROGRESS_INTERVAL, JobHandle};
pub use crate::job_outcome::JobOutcome;
pub use crate::job_progress::JobProgress;
pub use crate::job_scope::JobScope;
pub use crate::progress_level::ProgressLevel;
pub use crate::progress_sink::{NoopSink, ProgressSink, RecordingSink};
pub use crate::progress_unit::ProgressUnit;
