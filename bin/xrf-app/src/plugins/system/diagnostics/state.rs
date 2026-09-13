use std::sync::{Mutex, MutexGuard};

use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::core::types::TauriResult;

/// One retained reader of the machine, refreshed on the way into every call that borrows it.
///
/// Held rather than built per call because a fresh reader rediscovers every process on the machine, and a surface
/// showing memory polls this. Behind a `Mutex` because refreshing takes `&mut` and commands are concurrent.
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
  ///
  /// Lending rather than exposing the reader keeps the refresh policy with the state that owns it: what a snapshot is
  /// allowed to read and what it pays to read it are the same decision, and two commands answering off different
  /// refreshes would disagree about the same machine.
  ///
  /// Every process rather than this one, because the webview is a separate process on this stack and the only way to
  /// find it is to look at what descends from here. Memory only, of both kinds: nothing here reads a command line, an
  /// environment, a user, or a swap figure, and each of those is a syscall per process on Windows.
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
