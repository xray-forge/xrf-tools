use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_app_core::TauriResult;
use xrf_job::PoolOnceLock;

use crate::visuals::SpawnVisual;

/// One visual's slot: empty until read, then the visual or its failure. The renderer's loaders are pool workers, so one
/// waiting on another's read keeps working the pool rather than blocking it.
type Cell = Arc<PoolOnceLock<TauriResult<Arc<SpawnVisual>>>>;

/// A session's spawned visuals, each read on first ask and kept.
pub(crate) struct SpawnVisuals {
  cells: Mutex<HashMap<String, Cell>>,
}

impl SpawnVisuals {
  pub(crate) fn new() -> Self {
    Self {
      cells: Mutex::new(HashMap::new()),
    }
  }

  /// The visual `name`, read by `read` on first ask; a concurrent ask waits for that read.
  ///
  /// # Errors
  ///
  /// Returns why the visual cannot be read, as every later ask does too.
  pub(crate) fn get_or_read(
    &self,
    name: &str,
    read: impl FnOnce() -> TauriResult<Arc<SpawnVisual>> + Send,
  ) -> TauriResult<Arc<SpawnVisual>> {
    let cell: Cell = Arc::clone(self.lock().entry(name.to_owned()).or_default());

    cell.get_or_init(read).clone()
  }

  /// The cells, whole even after a panic: the lock guards only inserting an empty cell.
  fn lock(&self) -> MutexGuard<'_, HashMap<String, Cell>> {
    self.cells.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
