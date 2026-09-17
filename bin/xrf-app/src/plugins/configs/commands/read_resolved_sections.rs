use std::sync::Arc;

use tauri::State;
use xrf_ltx_inspect::LtxResolvedSection;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsReadSectionsRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// How many sections one request may ask for.
/// todo: Vertically tuned wide screen?
const MAXIMUM_SECTIONS_PER_READ: usize = 64;

/// Reads the bodies of the named sections of one entry point.
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

  let opened: Arc<SessionSnapshot<ConfigsProject>> = state.require(session_id)?;
  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;
  execution
    .run_blocking("Configs resolved page", move || {
      let names: Vec<&str> = names.iter().map(String::as_str).collect::<Vec<&str>>();

      opened.with_reader(&entry, true, |reader, resolved| {
        reader
          .read_sections(&names)
          .map_err(|error| format!("Cannot read sections of '{}': {error}", resolved.entry.as_str()))
      })
    })
    .await?
}
