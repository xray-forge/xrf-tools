use std::sync::Arc;

use tauri::State;
use xrf_ltx_inspect::LtxResolvedIndex;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsResolvedRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// Lists every section one entry point resolves to, named and counted.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_resolved_sections"))]
#[tauri::command(rename = "list_resolved_sections")]
pub async fn configs_list_resolved_sections(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsResolvedRequest,
) -> TauriResult<Arc<LtxResolvedIndex>> {
  let ConfigsResolvedRequest { session_id, entry } = request;

  let opened: Arc<SessionSnapshot<ConfigsProject>> = state.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  execution
    .run_blocking("Configs resolved index", move || {
      opened.with_reader(&entry, true, |reader, resolved| {
        if let Some(index) = resolved.get_index()? {
          return Ok(index);
        }

        let index: Arc<LtxResolvedIndex> = Arc::new(
          reader
            .read_index()
            .map_err(|error| format!("Cannot index '{}': {error}", resolved.entry.as_str()))?,
        );

        resolved.hold_index(Arc::clone(&index))?;

        Ok(index)
      })
    })
    .await?
}
