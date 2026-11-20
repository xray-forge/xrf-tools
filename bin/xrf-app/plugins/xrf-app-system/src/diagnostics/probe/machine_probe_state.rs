use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System};
use xrf_app_core::TauriResult;

/// One retained reader of the machine, refreshed on the way into every call that borrows it.
pub struct MachineProbeState {
  /// Opened by the first reading, so a machine that cannot be read fails that reading rather than the application's
  /// start, and a later reading tries again.
  system: Arc<Mutex<Option<System>>>,
}

impl Default for MachineProbeState {
  fn default() -> Self {
    Self::new()
  }
}

impl MachineProbeState {
  /// A reader that has read nothing yet.
  pub fn new() -> Self {
    Self {
      system: Arc::new(Mutex::new(None)),
    }
  }

  /// Refreshes what a usage reading needs and lends the reader to `read`, on a blocking thread: the refresh walks every
  /// process of the machine, which no async worker should wait through.
  ///
  /// # Errors
  ///
  /// Returns an error when the machine cannot be read, or when the blocking thread cannot run the reading.
  pub async fn read<T: Send + 'static>(&self, read: impl FnOnce(&System) -> T + Send + 'static) -> TauriResult<T> {
    let system: Arc<Mutex<Option<System>>> = Arc::clone(&self.system);

    tauri::async_runtime::spawn_blocking(move || -> TauriResult<T> {
      let mut reader: MutexGuard<'_, Option<System>> = Self::lock(&system);
      let system: &mut System = Self::open(&mut reader)?;

      system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory(),
      );
      system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

      Ok(read(system))
    })
    .await
    .map_err(|error| format!("The machine could not be read: {error}"))?
  }

  /// The retained reader, opened here by the first reading that finds none.
  fn open(reader: &mut Option<System>) -> TauriResult<&mut System> {
    match reader {
      Some(system) => Ok(system),
      empty => Ok(empty.insert(System::new().map_err(|error| format!("The machine could not be read: {error}"))?)),
    }
  }

  // Every reading refreshes the reader whole first, so one a panic left half refreshed is refreshed again.
  fn lock(system: &Mutex<Option<System>>) -> MutexGuard<'_, Option<System>> {
    system.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
