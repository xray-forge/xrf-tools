use std::sync::Arc;

use tauri::State;
use xrf_ltx_inspect::LtxResolvedIndex;
use xrf_vfs::XrayLogicalPath;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsResolvedRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// Lists every section one entry point resolves to, named and counted.
///
/// The index, not the bodies: a vanilla `system.ltx` resolves to 23,500 sections holding 293,000 fields, and sending
/// those together would be a message of tens of megabytes for a screen showing forty lines. What travels is enough to
/// lay the document out - how many fields each section has, so the view knows its own height - and bodies are asked
/// for a page at a time as they scroll into view.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_resolved_sections"))]
#[tauri::command(rename = "list_resolved_sections")]
pub async fn configs_list_resolved_sections(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsResolvedRequest,
) -> TauriResult<Arc<LtxResolvedIndex>> {
  let ConfigsResolvedRequest { session_id, entry } = request;

  let opened: Arc<SessionSnapshot<ConfigsProject>> = state.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(|error| error.to_string())?;

  // Off the async worker: the first ask for a root resolves it, then walks every section and reads back the header of
  // every config declaring one.
  execution
    .run_blocking("Configs resolved index", move || {
      opened.with_reader(&entry, |reader, resolved| {
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
