use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_app_core::types::TauriResult;
use xrf_ltx_inspect::LtxSectionSchemeReport;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::configs_project::ConfigsProject;
use crate::request::ConfigsSectionRequest;
use crate::state::ConfigsState;

/// What one section is judged by, and how it measures against that.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_section_scheme"))]
#[tauri::command(rename = "read_section_scheme")]
pub async fn configs_read_section_scheme(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsSectionRequest,
) -> TauriResult<Option<LtxSectionSchemeReport>> {
  let ConfigsSectionRequest {
    session_id,
    entry,
    section,
  } = request;

  let opened: Arc<SessionSnapshot<ConfigsProject>> = state.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  execution
    .run_blocking("Configs section scheme", move || {
      opened.with_reader(&entry, true, |reader, _| Ok(reader.read_section_scheme(&section)))
    })
    .await?
}
