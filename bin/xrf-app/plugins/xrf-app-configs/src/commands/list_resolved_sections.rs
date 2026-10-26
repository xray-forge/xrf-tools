use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_ltx_inspect::LtxResolvedIndex;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::project::ConfigsProjects;
use crate::request::ConfigsListRequest;

/// Lists every section one entry point resolves to, named and counted.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "list_resolved_sections")]
pub async fn configs_list_resolved_sections(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsListRequest,
) -> TauriResult<SessionSnapshot<Arc<LtxResolvedIndex>>> {
  let ConfigsListRequest { session_id, entry } = request;

  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  projects
    .read_on_pool(&execution, session_id, "Configs resolved index", move |opened| {
      opened.index(&entry)
    })
    .await
}
