use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_ltx_inspect::LtxDeclarationQuery;

use crate::project::{ConfigsDeclarationSearch, ConfigsProjects};
use crate::request::ConfigsFindDeclarationsRequest;

/// Where the open project writes a section, a field or a value: declarations and patches alike, archived configs too.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "find_declarations")]
pub async fn configs_find_declarations(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsFindDeclarationsRequest,
) -> TauriResult<SessionSnapshot<ConfigsDeclarationSearch>> {
  let ConfigsFindDeclarationsRequest {
    session_id,
    section,
    field,
    value,
  } = request;

  if section.is_none() && field.is_none() && value.is_none() {
    return Err(String::from(
      "Cannot find declarations without a section, a field or a value to look for",
    ));
  }

  projects
    .read_on_pool(&execution, session_id, "Configs declarations", move |opened| {
      opened.find_declarations(LtxDeclarationQuery {
        section,
        field,
        value,
        ..LtxDeclarationQuery::default()
      })
    })
    .await
}
