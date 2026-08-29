use std::sync::{Arc, MutexGuard};

use tauri::State;
use xrf_archive::ArchiveProject;

use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub fn archives_close_project(state: State<'_, ArchiveProjectState>) -> TauriResult {
  log::info!("Closing archives project");

  let mut lock: MutexGuard<Option<Arc<ArchiveProject>>> = state.project.lock().unwrap();

  if lock.is_some() {
    *lock = None;
  }

  Ok(())
}
