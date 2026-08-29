use std::sync::{Arc, MutexGuard};

use tauri::State;
use xrf_archive::ArchiveProject;

use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn archives_get_project(state: State<'_, ArchiveProjectState>) -> TauriResult<Option<Arc<ArchiveProject>>> {
  log::debug!("Getting archives project");

  let lock: MutexGuard<Option<Arc<ArchiveProject>>> = state.project.lock().unwrap();

  Ok(lock.clone())
}
