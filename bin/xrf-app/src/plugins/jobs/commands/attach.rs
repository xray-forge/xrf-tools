use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::JobProgress;

use crate::core::jobs::JobRegistry;

/// Watch a running job this window did not start.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "attach"))]
#[tauri::command(rename = "attach")]
pub fn jobs_attach(registry: State<'_, Arc<JobRegistry>>, id: Uuid, progress: Channel<JobProgress>) -> bool {
  log::info!("Attaching to job: {}", id);

  registry.attach(id, progress)
}
