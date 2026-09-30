use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

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
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned.
  pub fn expect(&self, visuals: &[String]) -> Result<(), String> {
    self.lock()?.pending = visuals.iter().cloned().collect();

    Ok(())
  }

  /// The estimator, built by `build` where it has not been or has been let go; a failure is kept, not tried again.
  ///
  /// # Errors
  ///
  /// Returns why the estimator cannot be built, or that the lock is poisoned.
  pub fn get_or_build(
    &self,
    build: impl FnOnce() -> Result<Arc<HemiEstimator>, String>,
  ) -> Result<Arc<HemiEstimator>, String> {
    let mut held: MutexGuard<Held> = self.lock()?;

    held.estimator.get_or_insert_with(build).clone()
  }

  /// Notes visuals described, and lets the estimator go once none expected is left.
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned.
  pub fn note_described(&self, names: &[String]) -> Result<(), String> {
    let mut held: MutexGuard<Held> = self.lock()?;
    let was_pending: bool = !held.pending.is_empty();

    for name in names {
      held.pending.remove(name);
    }

    if was_pending && held.pending.is_empty() {
      held.estimator = None;
    }

    Ok(())
  }

  /// Whether an estimator is held, which is the memory it costs.
  #[cfg(test)]
  pub fn is_held(&self) -> bool {
    self.held.lock().is_ok_and(|held| matches!(held.estimator, Some(Ok(_))))
  }

  fn lock(&self) -> Result<MutexGuard<'_, Held>, String> {
    self
      .held
      .lock()
      .map_err(|error| format!("The level's spawned objects' lighting is unavailable: {error}"))
  }
}
