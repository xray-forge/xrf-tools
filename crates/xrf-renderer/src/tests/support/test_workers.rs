use std::sync::Arc;

use xrf_job::testing::create_test_pool;

use crate::thread::queue::render_workers::RenderWorkers;

/// A small pool of its own for a test's loaders, as the application hands the renderer its pool.
pub(crate) fn create_workers() -> RenderWorkers {
  RenderWorkers::new(Arc::new(create_test_pool(2)))
}
