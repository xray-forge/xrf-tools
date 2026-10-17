use std::sync::Arc;

use rayon::ThreadPool;

use crate::thread::frame_task::FrameTask;

/// The pool the renderer's loaders read and decode on: the application's own, handed in, so a level opening beside
/// other work shares one width rather than adding a pool the size of the machine.
#[derive(Clone)]
pub struct RenderWorkers {
  pool: Arc<ThreadPool>,
}

impl RenderWorkers {
  pub fn new(pool: Arc<ThreadPool>) -> Self {
    Self { pool }
  }

  /// Runs `work` on the pool, returning at once; whatever parallel work it does runs on the same pool.
  pub fn spawn(&self, work: impl FnOnce() + Send + 'static) {
    self.pool.spawn(work);
  }

  /// Offers work a frame waits for to the pool, which the thread joining it runs itself where no worker took it yet.
  pub(crate) fn spawn_frame_task<T: Send + 'static>(
    &self,
    work: impl FnOnce(bool) -> T + Send + 'static,
  ) -> FrameTask<T> {
    FrameTask::start(&self.pool, work)
  }
}
