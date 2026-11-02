use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_ltx_inspect::LtxAnchoredFinding;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::project::ConfigsProjects;
use crate::request::ConfigsListRequest;

/// Everything wrong with one resolved root, anchored to the file and line a person has to open; verified once and
/// answered as held.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "list_findings")]
pub async fn configs_list_findings(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsListRequest,
) -> TauriResult<SessionSnapshot<Arc<Vec<LtxAnchoredFinding>>>> {
  let ConfigsListRequest { session_id, entry } = request;

  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  projects
    .read_on_pool(&execution, session_id, "Configs findings", move |opened| {
      opened.find_problems(&entry)
    })
    .await
}
