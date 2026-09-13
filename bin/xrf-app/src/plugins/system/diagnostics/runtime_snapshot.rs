//! What the application costs the machine at the moment it is asked.

use serde::Serialize;
use sysinfo::{Pid, Process, System};
use xrf_utils::duration_to_millis;

use crate::core::process::{process_started_at_epoch_millis, process_uptime};
use crate::plugins::system::diagnostics::process_tree::{DescendantUsage, sum_descendants};

/// One reading of what the application costs, and of how long it has been running.
///
/// Every figure is a reading rather than a total: nothing here accumulates, so a caller polling this sees the current
/// state and never a history it did not ask to keep.
///
/// Grouped by subject rather than flattened, because the same word means three different things depending on whose
/// memory is being reported, and a prefix on each field is a worse way of saying so than a name around each group.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
  /// Milliseconds since the epoch at which the process began, stamped in `main`.
  pub started_at: u64,
  /// Milliseconds it has been running, measured monotonically rather than by subtracting two wall-clock readings.
  pub uptime: u64,
  /// What the backend process itself holds.
  pub process: ProcessUsage,
  /// What the processes below it hold, which on this stack is the webview.
  pub descendants: DescendantUsage,
  /// What the whole machine is using, for reading the two above against.
  pub machine: MachineUsage,
}

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

impl RuntimeSnapshot {
  /// Reads one snapshot off a refreshed view of the machine.
  ///
  /// Takes the reader rather than acquiring one, so the refresh policy stays with the state that lends it.
  pub fn read(system: &System, pid: Pid) -> Self {
    Self {
      started_at: process_started_at_epoch_millis(),
      uptime: duration_to_millis(process_uptime()),
      process: ProcessUsage::of(system, pid),
      descendants: sum_descendants(system, pid),
      machine: MachineUsage::of(system),
    }
  }
}

impl ProcessUsage {
  /// What one process holds, or nothing where the table no longer lists it.
  fn of(system: &System, pid: Pid) -> Self {
    Self {
      resident_memory: system.process(pid).map_or(0, Process::memory),
      virtual_memory: system.process(pid).map_or(0, Process::virtual_memory),
    }
  }
}

impl MachineUsage {
  fn of(system: &System) -> Self {
    Self {
      used_memory: system.used_memory(),
      available_memory: system.available_memory(),
    }
  }
}
