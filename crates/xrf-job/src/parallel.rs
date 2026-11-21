//! The parallel work the workspace does, on whatever pool its caller installed.
//!
//! Rayon is this crate's dependency alone, so pools are built in one place: an [`crate::ExecutionPlan`] builds the
//! binaries' pools and `xrf_job::testing` (feature `testing`) the tests'. Everything else fans out through these items, which run on the
//! pool installed around the caller, and never builds a pool of its own.

pub use rayon::{ThreadPool, current_num_threads, current_thread_index, join, prelude, spawn};
