use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_ltx_inspect::LtxResolvedIndex;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::request::ConfigsResolvedRequest;
use crate::{ConfigsProject, ConfigsProjects};

/// Lists every section one entry point resolves to, named and counted.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_resolved_sections"))]
#[tauri::command(rename = "list_resolved_sections")]
pub async fn configs_list_resolved_sections(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsResolvedRequest,
) -> TauriResult<Arc<LtxResolvedIndex>> {
  let ConfigsResolvedRequest { session_id, entry } = request;

  let opened: Arc<SessionSnapshot<ConfigsProject>> = projects.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  execution
    .run_blocking("Configs resolved index", move || opened.index(&entry))
    .await?
}
