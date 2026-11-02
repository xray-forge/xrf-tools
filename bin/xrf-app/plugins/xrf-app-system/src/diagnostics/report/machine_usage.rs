use serde::Serialize;
use sysinfo::System;

/// What the machine as a whole is using.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineUsage {
  /// Physical memory in use across every process.
  pub used_memory: u64,
  /// Physical memory the machine reports as free for a new allocation, which is not `total - used`: the difference is
  /// cache the operating system would hand back under pressure.
  pub available_memory: u64,
}

impl MachineUsage {
  /// What the machine is using, off a refreshed view of it.
  pub fn of(system: &System) -> Self {
    Self {
      used_memory: system.used_memory(),
      available_memory: system.available_memory(),
    }
  }
}
