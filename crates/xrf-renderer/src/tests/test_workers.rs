use std::sync::Arc;

use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::thread::render_workers::RenderWorkers;

/// What a test's pool threads are named, so a test can tell its pool's work from the global pool's.
pub const TEST_WORKER_NAME: &str = "xrf-render-test-worker";

/// A small pool of its own for a test's loaders, as the application hands the renderer its pool.
pub fn create_workers() -> RenderWorkers {
  let pool: ThreadPool = ThreadPoolBuilder::new()
    .num_threads(2)
    .thread_name(|index| format!("{TEST_WORKER_NAME}-{index}"))
    .build()
    .expect("A test pool starts");

  RenderWorkers::new(Arc::new(pool))
}
