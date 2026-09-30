use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use crate::core::types::TauriResult;
use crate::plugins::levels::state::level_spawn_visual::LevelSpawnVisual;

/// One visual's place: empty until it is read, then the visual, or why it cannot be.
type Cell = Arc<OnceLock<Result<Arc<LevelSpawnVisual>, String>>>;

/// The visuals a level's spawned objects stand as, each read the first time anything asks and kept.
pub struct LevelSpawnVisuals {
  cells: Mutex<HashMap<String, Cell>>,
}

impl LevelSpawnVisuals {
  pub fn new() -> Self {
    Self {
      cells: Mutex::new(HashMap::new()),
    }
  }

  /// The visual by name, read by `read` the first time anything asks; a second ask made while it is read waits for it.
  ///
  /// # Errors
  ///
  /// Returns why the visual cannot be read, which every later ask answers with again, or that the lock is poisoned.
  pub fn get_or_read(
    &self,
    name: &str,
    read: impl FnOnce() -> Result<Arc<LevelSpawnVisual>, String>,
  ) -> Result<Arc<LevelSpawnVisual>, String> {
    let cell: Cell = Arc::clone(self.lock()?.entry(name.to_owned()).or_default());

    cell.get_or_init(read).clone()
  }

  /// The visual by name, where one was read.
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned.
  pub fn get(&self, name: &str) -> TauriResult<Option<Arc<LevelSpawnVisual>>> {
    Ok(self.lock()?.get(name).and_then(|cell| cell.get()?.clone().ok()))
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, HashMap<String, Cell>>> {
    self
      .cells
      .lock()
      .map_err(|error| format!("The level's spawned visuals are unavailable: {error}"))
  }
}
