#![doc = include_str!("../README.md")]

mod execution;
mod job;
pub mod parallel;
mod progress;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

#[cfg(test)]
mod tests;

pub use crate::execution::{ExecutionOrigin, ExecutionPlan, ExecutionRequest, PoolOnceLock};
pub use crate::job::{JobHandle, JobOutcome, JobScope};
pub use crate::progress::{JobProgress, NoopSink, ProgressLevel, ProgressSink, ProgressUnit, RecordingSink};
