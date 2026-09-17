use std::sync::Arc;

use tauri::State;
use uuid::Uuid;

use crate::core::jobs::JobRegistry;

/// Ask a running job to stop at its next safe boundary.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "cancel"))]
#[tauri::command(rename = "cancel")]
pub fn jobs_cancel(registry: State<'_, Arc<JobRegistry>>, id: Uuid) -> bool {
  log::info!("Cancelling job: {}", id);

  registry.cancel(id)
}
