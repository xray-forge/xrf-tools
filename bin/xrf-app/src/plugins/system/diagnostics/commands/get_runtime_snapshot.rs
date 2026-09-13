use sysinfo::Pid;
use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::system::diagnostics::{MachineProbeState, RuntimeSnapshot};

/// Report what the application currently costs the machine, and how long it has been running.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_runtime_snapshot"))]
#[tauri::command(rename = "get_runtime_snapshot")]
pub async fn system_get_runtime_snapshot(state: State<'_, MachineProbeState>) -> TauriResult<RuntimeSnapshot> {
  let pid: Pid = Pid::from_u32(std::process::id());

  state.with_reader(|system| RuntimeSnapshot::read(system, pid))
}
