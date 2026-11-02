use serde::Serialize;
use sysinfo::{Pid, Process, System};

/// What one process holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessUsage {
  /// Physical memory the process actually occupies.
  pub resident_memory: u64,
  /// Address space it has reserved, which is routinely several times the resident set and is not what it costs.
  pub virtual_memory: u64,
}

impl ProcessUsage {
  /// What one process holds, or nothing where the table no longer lists it.
  pub fn of(system: &System, pid: Pid) -> Self {
    Self {
      resident_memory: system.process(pid).map_or(0, Process::memory),
      virtual_memory: system.process(pid).map_or(0, Process::virtual_memory),
    }
  }
}
