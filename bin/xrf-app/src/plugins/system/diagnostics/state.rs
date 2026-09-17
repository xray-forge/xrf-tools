use std::sync::{Mutex, MutexGuard};

use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::core::types::TauriResult;

/// One retained reader of the machine, refreshed on the way into every call that borrows it.
pub struct MachineProbeState {
  system: Mutex<System>,
}

impl MachineProbeState {
  pub fn new() -> Self {
    Self {
      system: Mutex::new(System::new()),
    }
  }

  /// Refreshes what a usage reading needs and lends the reader for one call.
  pub fn with_reader<T>(&self, consumer: impl FnOnce(&System) -> T) -> TauriResult<T> {
    let mut system: MutexGuard<'_, System> = self
      .system
      .lock()
      .map_err(|_| String::from("The machine probe was poisoned by an earlier failure"))?;

    system.refresh_processes_specifics(
      ProcessesToUpdate::All,
      true,
      ProcessRefreshKind::nothing().with_memory(),
    );
    system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

    Ok(consumer(&system))
  }
}
