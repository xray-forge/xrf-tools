use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_export::{ExportsProject, ExportsProjectParser};
use xrf_utils::error_to_string;

use crate::ExportsProjects;

/// Open the scripts project at `project_path` and parse every extern it declares, answering the whole project, which
/// the page lists in full.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_project")]
pub async fn exports_open_project(
  session_id: SessionId,
  project_path: String,
  projects: State<'_, ExportsProjects>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<Arc<SessionSnapshot<ExportsProject>>> {
  log::info!("Parsing externs from project: {project_path}");

  projects
    .open_on_pool(&execution, session_id, "Opening an exports project", move || {
      ExportsProjectParser::new()
        .parse_project_from_path(&project_path)
        .map_err(error_to_string)
    })
    .await
}
