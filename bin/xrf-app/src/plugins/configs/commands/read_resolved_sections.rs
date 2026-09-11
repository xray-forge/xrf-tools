use std::sync::Arc;

use tauri::State;
use xrf_ltx_inspect::LtxResolvedSection;
use xrf_vfs::XrayLogicalPath;

use crate::core::execution::ExecutionState;
use crate::core::session::DocumentSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsReadSectionsRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// How many sections one request may ask for.
///
/// A page is what scrolled into view, not a slice of the document a caller chose the size of. A section is at least
/// two lines - its header and the gap after it - so this covers a window twice as tall as any screen the application
/// runs on; a request past it is a caller that stopped paging, and refusing it is better than quietly answering with a
/// message it did not expect.
///
/// todo: Vertically tuned wide screen?
const MAXIMUM_SECTIONS_PER_READ: usize = 64;

/// Reads the bodies of the named sections of one entry point.
///
/// Addressed by name rather than by offset, so a page is always whole sections and a filter applied on one side never
/// has to be mirrored on the other. A name the root does not hold is skipped rather than refused: a page request races
/// an index the caller may have fetched before a reopen.
///
/// Off the async worker even though a page is usually a map lookup: the root is normally resolved by the time anything
/// can ask for one, but "normally" is not a guarantee. A page asked for after the session was replaced would resolve a
/// whole include tree, and doing that on the IPC handler thread would stall every other command behind it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_resolved_sections"))]
#[tauri::command(rename = "read_resolved_sections")]
pub async fn configs_read_resolved_sections(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsReadSectionsRequest,
) -> TauriResult<Vec<LtxResolvedSection>> {
  let ConfigsReadSectionsRequest {
    session_id,
    entry,
    names,
  } = request;

  if names.len() > MAXIMUM_SECTIONS_PER_READ {
    return Err(format!(
      "Cannot read {} sections at once, {MAXIMUM_SECTIONS_PER_READ} is the most one page may ask for",
      names.len()
    ));
  }

  let opened: Arc<DocumentSnapshot<ConfigsProject>> = state.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(|error| error.to_string())?;
  execution
    .run_blocking("Configs resolved page", move || {
      let names: Vec<&str> = names.iter().map(String::as_str).collect::<Vec<&str>>();

      opened.with_reader(&entry, |reader, resolved| {
        reader
          .read_sections(&names)
          .map_err(|error| format!("Cannot read sections of '{}': {error}", resolved.entry.as_str()))
      })
    })
    .await?
}
