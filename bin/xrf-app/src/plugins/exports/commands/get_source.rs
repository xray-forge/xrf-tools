use tauri::State;
use xrf_export::ExportSourceContent;

use crate::core::error::error_to_string;
use crate::core::session::DocumentSessionId;
use crate::core::types::TauriResult;
use crate::plugins::exports::state::ExportsProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_source"))]
#[tauri::command(rename = "get_source")]
pub async fn exports_get_source(
  session_id: DocumentSessionId,
  name: &str,
  state: State<'_, ExportsProjectState>,
) -> TauriResult<ExportSourceContent> {
  log::info!("Reading source of xr export: {name}");

  let project = state.require(session_id)?;

  let source: ExportSourceContent = project.read_declaration_source(name).map_err(error_to_string)?;

  Ok(source)
}
