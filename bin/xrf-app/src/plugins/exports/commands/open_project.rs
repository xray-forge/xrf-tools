use std::sync::Arc;

use tauri::State;
use xrf_export::{ExportsProject, ExportsProjectParser};

use crate::core::error::error_to_string;
use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::exports::state::ExportsProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn exports_open_project(
  session_id: DocumentSessionId,
  project_path: &str,
  state: State<'_, ExportsProjectState>,
) -> TauriResult<Arc<DocumentSnapshot<ExportsProject>>> {
  state.begin_open(session_id)?;
  log::info!("Parsing externs from project: {project_path}");

  let parser: ExportsProjectParser = ExportsProjectParser::new();
  let project: ExportsProject = parser.parse_project_from_path(project_path).map_err(error_to_string)?;
  state.commit_open(session_id, project)
}
