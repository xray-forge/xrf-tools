use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System};
use xrf_app_core::TauriResult;

/// One retained reader of the machine, refreshed on the way into every call that borrows it.
pub struct MachineProbeState {
  system: Arc<Mutex<System>>,
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
      system: Arc::new(Mutex::new(System::new())),
    }
  }

  /// Refreshes what a usage reading needs and lends the reader to `read`, on a blocking thread: the refresh walks every
  /// process of the machine, which no async worker should wait through.
  ///
  /// # Errors
  ///
  /// Returns an error when the blocking thread cannot run the reading.
  pub async fn read<T: Send + 'static>(&self, read: impl FnOnce(&System) -> T + Send + 'static) -> TauriResult<T> {
    let system: Arc<Mutex<System>> = Arc::clone(&self.system);

    tauri::async_runtime::spawn_blocking(move || {
      let mut system: MutexGuard<'_, System> = Self::lock(&system);

      system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory(),
      );
      system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

      read(&system)
    })
    .await
    .map_err(|error| format!("The machine could not be read: {error}"))
  }

  // Every reading refreshes the reader whole first, so one a panic left half refreshed is refreshed again.
  fn lock(system: &Mutex<System>) -> MutexGuard<'_, System> {
    system.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
