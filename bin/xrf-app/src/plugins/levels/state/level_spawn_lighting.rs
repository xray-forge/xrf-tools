use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_visual::HemiEstimator;

/// What the lighting holds between batches: the estimator once built, and the visuals still to be described.
#[derive(Default)]
struct Held {
  estimator: Option<Result<Arc<HemiEstimator>, String>>,
  pending: HashSet<String>,
}

/// How a level lights its spawned objects while their models are read. The estimator holds the whole collision form,
/// about a hundred megabytes on a large level, and nothing asks for it once every visual is described, so it is built
/// the first time a batch asks and let go after the last.
pub struct LevelSpawnLighting {
  held: Mutex<Held>,
}

impl LevelSpawnLighting {
  pub fn new() -> Self {
    Self {
      held: Mutex::new(Held::default()),
    }
  }

  /// Expects every visual the viewer draws to be described, the estimator kept until they have been.
  pub fn expect(&self, visuals: &[String]) {
    self.lock().pending = visuals.iter().cloned().collect();
  }

  /// The estimator, built by `build` where it has not been or has been let go; a failure is kept, not tried again.
  ///
  /// # Errors
  ///
  /// Returns why the estimator cannot be built.
  pub fn get_or_build(
    &self,
    build: impl FnOnce() -> Result<Arc<HemiEstimator>, String>,
  ) -> Result<Arc<HemiEstimator>, String> {
    let mut held: MutexGuard<Held> = self.lock();

    held.estimator.get_or_insert_with(build).clone()
  }

  /// Notes visuals described, and lets the estimator go once none expected is left: after the last, and after a batch
  /// asked again once none was, which built it anew.
  pub fn note_described(&self, names: &[String]) {
    let mut held: MutexGuard<Held> = self.lock();

    for name in names {
      held.pending.remove(name);
    }

    if held.pending.is_empty() {
      held.estimator = None;
    }
  }

  /// Whether an estimator is held, which is the memory it costs.
  #[cfg(test)]
  pub fn is_held(&self) -> bool {
    matches!(self.lock().estimator, Some(Ok(_)))
  }

  /// The state, taken back from a build that panicked holding it: it is a cache and a set of names, either of which a
  /// later batch makes good, rather than lighting lost for the rest of the level.
  fn lock(&self) -> MutexGuard<'_, Held> {
    self.held.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
