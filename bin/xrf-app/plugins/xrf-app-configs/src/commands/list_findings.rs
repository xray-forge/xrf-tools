use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_ltx_inspect::LtxAnchoredFinding;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::request::ConfigsResolvedRequest;
use crate::{ConfigsProject, ConfigsProjects};

/// Everything wrong with one resolved root, anchored to the file and line a person has to open; verified once and
/// answered as held.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_findings"))]
#[tauri::command(rename = "list_findings")]
pub async fn configs_list_findings(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsResolvedRequest,
) -> TauriResult<Arc<Vec<LtxAnchoredFinding>>> {
  let ConfigsResolvedRequest { session_id, entry } = request;

  let opened: Arc<SessionSnapshot<ConfigsProject>> = projects.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  execution
    .run_blocking("Configs findings", move || opened.find_problems(&entry))
    .await?
}
