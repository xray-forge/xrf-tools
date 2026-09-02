use std::num::NonZeroUsize;

use serde::Serialize;

#[cfg(feature = "rayon")]
use rayon::{ThreadPool, ThreadPoolBuilder};
#[cfg(feature = "rayon")]
use xrf_error::{XrfError, XrfResult};

use crate::execution_origin::ExecutionOrigin;

/// How much of the machine one operation may use, and whether anybody chose it.
///
/// Resolved, never a preference: [`crate::ExecutionRequest::resolve`] is the only way to obtain one, so an operation
/// receiving a plan is receiving a decision that has already been made rather than a question it has to answer. That is
/// what lets a report state the width a run actually used instead of the words a command line happened to carry.
///
/// The count is a ceiling on the whole operation, not a suggestion per call site. A caller bounds itself by installing
/// the plan's pool once, around everything; work nested inside inherits it, so a count means what it says even when the
/// work fans out several levels deep.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionPlan {
  workers: NonZeroUsize,
  origin: ExecutionOrigin,
}

impl ExecutionPlan {
  pub(crate) fn new(workers: NonZeroUsize, origin: ExecutionOrigin) -> Self {
    Self { workers, origin }
  }

  /// The most workers this operation may run at once. One is a real sequential run.
  pub fn get_workers(&self) -> NonZeroUsize {
    self.workers
  }

  pub fn get_origin(&self) -> ExecutionOrigin {
    self.origin
  }
}

#[cfg(feature = "rayon")]
impl ExecutionPlan {
  /// Builds the pool this plan describes.
  ///
  /// For a caller that keeps one pool across several operations — a desktop application running concurrent jobs, where
  /// a pool per job would multiply the plan by the job count and leave nothing bounding the total.
  ///
  /// # Errors
  ///
  /// Returns an error when the threads cannot be started.
  pub fn build_pool(&self) -> XrfResult<ThreadPool> {
    ThreadPoolBuilder::new()
      .num_threads(self.get_workers().get())
      .thread_name(|index| format!("xrf-worker-{index}"))
      .build()
      .map_err(|error| {
        XrfError::new_unexpected_error(format!("cannot start {} worker(s): {error}", self.get_workers()))
      })
  }

  /// Runs `operation` inside this plan's pool, so everything it reaches is bounded by the plan.
  ///
  /// For a caller running one operation per process. Parallel work nested anywhere inside — including inside a
  /// dependency that reaches for Rayon on its own — runs on this pool rather than the global one, which is what makes
  /// the count an upper bound rather than a hint honoured only where somebody remembered to.
  ///
  /// # Errors
  ///
  /// Returns an error when the threads cannot be started. `operation` does not run in that case.
  pub fn install<R, F>(&self, operation: F) -> XrfResult<R>
  where
    F: FnOnce() -> R + Send,
    R: Send,
  {
    Ok(self.build_pool()?.install(operation))
  }
}
