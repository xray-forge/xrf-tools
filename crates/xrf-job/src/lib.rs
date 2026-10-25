#![doc = include_str!("../README.md")]

mod execution;
mod job;
mod progress;

#[cfg(test)]
mod tests;

#[cfg(feature = "rayon")]
pub use crate::execution::PoolOnceLock;
pub use crate::execution::{ExecutionOrigin, ExecutionPlan, ExecutionRequest};
pub use crate::job::{JobHandle, JobOutcome, JobScope};
pub use crate::progress::{JobProgress, NoopSink, ProgressLevel, ProgressSink, ProgressUnit, RecordingSink};
