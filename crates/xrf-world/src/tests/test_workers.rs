use std::sync::Arc;

use rayon::{ThreadPool, ThreadPoolBuilder};
use xrf_renderer::RenderWorkers;

/// A small pool of its own for a test's loaders, as the application hands the world its pool.
pub fn create_workers() -> RenderWorkers {
  let pool: ThreadPool = ThreadPoolBuilder::new()
    .num_threads(2)
    .thread_name(|index| format!("xrf-world-test-worker-{index}"))
    .build()
    .expect("A test pool starts");

  RenderWorkers::new(Arc::new(pool))
}
