//! What the application costs the machine at the moment it is asked.

use serde::Serialize;
use sysinfo::{Pid, System};
use xrf_app_core::logging::ProcessStart;
use xrf_utils::duration_to_millis;

use crate::diagnostics::report::{DescendantUsage, MachineUsage, ProcessUsage};

/// One reading of what the application costs, and of how long it has been running.
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

impl RuntimeSnapshot {
  /// Reads one snapshot off a refreshed view of the machine.
  pub fn read(system: &System, pid: Pid) -> Self {
    Self {
      started_at: ProcessStart::get_started_at(),
      uptime: duration_to_millis(ProcessStart::get_uptime()),
      process: ProcessUsage::of(system, pid),
      descendants: DescendantUsage::sum(system, pid),
      machine: MachineUsage::of(system),
    }
  }
}
