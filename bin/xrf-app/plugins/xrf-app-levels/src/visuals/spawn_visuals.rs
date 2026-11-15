use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_app_core::TauriResult;
use xrf_job::PoolOnceLock;

use crate::visuals::SpawnVisual;

/// One visual's place: empty until it is read, then the visual, or why it cannot be. Filled from the renderer's loaders,
/// which are pool workers, so one waiting on another's read keeps working the pool rather than blocking it.
type Cell = Arc<PoolOnceLock<TauriResult<Arc<SpawnVisual>>>>;

/// The visuals spawned objects stand as, each read the first time anything asks and kept: a level's spawn, or a
/// scene's spawn points, each keeping its own.
pub(crate) struct SpawnVisuals {
  cells: Mutex<HashMap<String, Cell>>,
}

impl SpawnVisuals {
  /// No visual read yet.
  pub(crate) fn new() -> Self {
    Self {
      cells: Mutex::new(HashMap::new()),
    }
  }

  /// The visual by name, read by `read` the first time anything asks; a second ask made while it is read waits for it.
  ///
  /// # Errors
  ///
  /// Returns why the visual cannot be read, which every later ask answers with again.
  pub(crate) fn get_or_read(
    &self,
    name: &str,
    read: impl FnOnce() -> TauriResult<Arc<SpawnVisual>> + Send,
  ) -> TauriResult<Arc<SpawnVisual>> {
    let cell: Cell = Arc::clone(self.lock().entry(name.to_owned()).or_default());

    cell.get_or_init(read).clone()
  }

  /// The cells, whole even after a panic under the lock: all it guards is the insert of an empty cell.
  fn lock(&self) -> MutexGuard<'_, HashMap<String, Cell>> {
    self.cells.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
