use std::sync::Arc;

use tauri::State;
use uuid::Uuid;

use crate::core::jobs::JobRegistry;

/// Ask a running job to stop at its next safe boundary.
///
/// Cooperative, so this returns as soon as the job has been told rather than once it has stopped: an operation
/// mid-write finishes that write, and the gap is visible in the listing as a job asked to stop but still running.
///
/// Answers whether anything is now expected to stop. `false` means the job has already finished — which is not a
/// failure, only the answer to a control the user pressed a moment too late.
///
/// A cancel for a job the registry has not seen is held rather than refused. The frontend knows a job's identity
/// before the command carrying it is sent, so a cancel can legitimately arrive first.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "cancel"))]
#[tauri::command(rename = "cancel")]
pub fn jobs_cancel(registry: State<'_, Arc<JobRegistry>>, id: Uuid) -> bool {
  log::info!("Cancelling job: {}", id);

  registry.cancel(id)
}
