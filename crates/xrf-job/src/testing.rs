//! Pools for tests, which build theirs here like every other pool rather than reaching for Rayon themselves.

use rayon::{ThreadPool, ThreadPoolBuilder};

/// What a test pool's threads are named, so a test can tell its pool's work from another thread's.
pub const TEST_WORKER_NAME: &str = "xrf-test-worker";

/// A pool of `workers` threads, named `TEST_WORKER_NAME-<index>`.
///
/// # Panics
///
/// Panics when the threads cannot be started, which fails the test that asked.
pub fn create_test_pool(workers: usize) -> ThreadPool {
  ThreadPoolBuilder::new()
    .num_threads(workers)
    .thread_name(|index| format!("{TEST_WORKER_NAME}-{index}"))
    .build()
    .expect("a test pool starts")
}
