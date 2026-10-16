use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use xrf_renderer::RenderWorkers;

/// How many of a loader's jobs started and how many finished, so that no more than the pool's width of them run or wait
/// on it at once: what other work queued behind them waits for stays one round of them, not a whole level. It bounds
/// the jobs, not the answers waiting to be taken.
///
/// Its owner starts the next jobs from the render thread as it takes answers, once a frame, rather than each job
/// starting the next as it ends: a job spawned from a pool thread goes to that thread's own queue, which the pool runs
/// before anything queued from outside, so the loader would again hold every other job back for the whole level.
pub struct LoaderFlight {
  bound: u32,
  started: u32,
  finished: Arc<AtomicU32>,
}

impl LoaderFlight {
  pub fn new(workers: &RenderWorkers) -> Self {
    Self {
      bound: u32::try_from(workers.get_width()).unwrap_or(u32::MAX).max(1),
      started: 0,
      finished: Arc::new(AtomicU32::new(0)),
    }
  }

  /// Counts one more job started, answering the count it adds itself to once finished, whatever it did; none while
  /// the pool's width of them are unfinished.
  pub fn start(&mut self) -> Option<Arc<AtomicU32>> {
    if self.started - self.finished.load(Ordering::Acquire) >= self.bound {
      return None;
    }

    self.started += 1;

    Some(Arc::clone(&self.finished))
  }

  /// How many jobs it started.
  pub fn get_started(&self) -> u32 {
    self.started
  }

  /// Whether every job it started has finished.
  pub fn is_landed(&self) -> bool {
    self.finished.load(Ordering::Acquire) == self.started
  }
}
