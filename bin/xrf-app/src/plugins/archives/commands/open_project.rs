use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_archive::ArchiveProject;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn archives_open_project(
  execution: State<'_, ExecutionState>,
  path: &str,
  state: State<'_, ArchiveProjectState>,
) -> TauriResult<Arc<ArchiveProject>> {
  log::info!("Opening archives project");

  let source: PathBuf = PathBuf::from(path);

  // Off the async worker: opening walks the directory and reads every volume's whole name table, which is work bounded
  // by the installation rather than by anything short enough for an IPC executor to hold.
  let project: Arc<ArchiveProject> = execution
    .run_blocking("Opening the archive project", move || ArchiveProject::new(&source))
    .await?
    .map(Arc::new)
    .map_err(|error| format!("Failed to open provided archive project: {}", error))?;

  log::info!("Opened archives project");

  // Shared rather than copied: the frontend and the state hold the same index, where storing it used to clone every
  // entry a second time.
  *state.project.lock().unwrap() = Some(project.clone());

  Ok(project)
}
