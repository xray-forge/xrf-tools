use std::sync::Arc;

use xrf_job::testing::create_test_pool;
use xrf_renderer::RenderWorkers;

/// A small pool of its own for a test's loaders, as the application hands the world its pool.
pub(crate) fn create_workers() -> RenderWorkers {
  RenderWorkers::new(Arc::new(create_test_pool(2)))
}
