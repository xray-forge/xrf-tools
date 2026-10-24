use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionId;
use xrf_export::ExportSourceContent;
use xrf_utils::error_to_string;

use crate::ExportsProjects;

/// Read the source the extern `name` is declared in, from the project `session_id` holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_source"))]
#[tauri::command(rename = "get_source")]
pub async fn exports_get_source(
  session_id: SessionId,
  name: &str,
  projects: State<'_, ExportsProjects>,
) -> TauriResult<ExportSourceContent> {
  log::info!("Reading source of xr export: {name}");

  projects
    .require(session_id)?
    .read_declaration_source(name)
    .map_err(error_to_string)
}
