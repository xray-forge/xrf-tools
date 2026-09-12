use std::sync::Arc;

use tauri::State;
use xrf_ltx_inspect::LtxSectionSchemeReport;
use xrf_vfs::XrayLogicalPath;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsSectionRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// What one section is judged by, and how it measures against that.
///
/// `None` means the root does not hold the section, which a panel reaches by asking about a selection the index no
/// longer holds.
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
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(|error| error.to_string())?;

  // Off the async worker for the same reason a page read is: the root is normally resolved by the time a section can
  // be selected, but a selection surviving a reopen would resolve a whole include tree on the IPC handler thread.
  execution
    .run_blocking("Configs section scheme", move || {
      opened.with_reader(&entry, |reader, _| Ok(reader.read_section_scheme(&section)))
    })
    .await?
}
