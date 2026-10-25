use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_ltx_inspect::LtxSectionSchemeReport;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::ConfigsProjects;
use crate::request::ConfigsReadSectionSchemeRequest;

/// What one section is judged by, and how it measures against that.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_section_scheme"))]
#[tauri::command(rename = "read_section_scheme")]
pub async fn configs_read_section_scheme(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsReadSectionSchemeRequest,
) -> TauriResult<SessionSnapshot<Option<LtxSectionSchemeReport>>> {
  let ConfigsReadSectionSchemeRequest {
    session_id,
    entry,
    section,
  } = request;

  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  projects
    .read_on_pool(&execution, session_id, "Configs section scheme", move |opened| {
      opened.with_reader(&entry, true, |reader, _| Ok(reader.read_section_scheme(&section)))
    })
    .await
}
