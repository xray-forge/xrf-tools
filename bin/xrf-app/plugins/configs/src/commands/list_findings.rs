use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_app_core::types::TauriResult;
use xrf_ltx_inspect::LtxAnchoredFinding;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::configs_project::ConfigsProject;
use crate::request::ConfigsResolvedRequest;
use crate::state::ConfigsState;

/// Everything wrong with one resolved root, anchored to the file and line a person has to open.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_findings"))]
#[tauri::command(rename = "list_findings")]
pub async fn configs_list_findings(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsResolvedRequest,
) -> TauriResult<Vec<LtxAnchoredFinding>> {
  let ConfigsResolvedRequest { session_id, entry } = request;

  let opened: Arc<SessionSnapshot<ConfigsProject>> = state.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  execution
    .run_blocking("Configs findings", move || {
      opened.find_problems(&entry).map(|findings| findings.as_ref().clone())
    })
    .await?
}
