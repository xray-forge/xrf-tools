use std::sync::Arc;

use tauri::State;
use xrf_ltx_inspect::LtxAnchoredFinding;
use xrf_vfs::XrayLogicalPath;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsResolvedRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// Everything wrong with one resolved root, anchored to the file and line a person has to open.
///
/// Held with the resolution, so opening the panel a second time is a lookup rather than a second walk of every section
/// the root holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_findings"))]
#[tauri::command(rename = "list_findings")]
pub async fn configs_list_findings(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsResolvedRequest,
) -> TauriResult<Vec<LtxAnchoredFinding>> {
  let ConfigsResolvedRequest { session_id, entry } = request;

  let opened: Arc<SessionSnapshot<ConfigsProject>> = state.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(|error| error.to_string())?;

  // Bounded by the root: verifying it walks every section it holds, which on a game tree is tens of thousands.
  execution
    .run_blocking("Configs findings", move || {
      opened.find_problems(&entry).map(|findings| findings.as_ref().clone())
    })
    .await?
}
