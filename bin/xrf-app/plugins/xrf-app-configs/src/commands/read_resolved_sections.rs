use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_ltx_inspect::LtxResolvedSection;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::ConfigsProjects;
use crate::request::ConfigsReadSectionsRequest;

/// How many sections one request may ask for: a guard on the request's size, since the page pages by its own row count.
const MAXIMUM_SECTIONS_PER_READ: usize = 64;

/// Reads the bodies of the named sections of one entry point.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_resolved_sections"))]
#[tauri::command(rename = "read_resolved_sections")]
pub async fn configs_read_resolved_sections(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsReadSectionsRequest,
) -> TauriResult<SessionSnapshot<Vec<LtxResolvedSection>>> {
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

  let entry: XrayLogicalPath = XrayLogicalPath::new(&entry).map_err(error_to_string)?;

  projects
    .read_on_pool(&execution, session_id, "Configs resolved page", move |opened| {
      let names: Vec<&str> = names.iter().map(String::as_str).collect::<Vec<&str>>();

      opened.with_reader(&entry, true, |reader, resolved| {
        reader
          .read_sections(&names)
          .map_err(|error| format!("Cannot read sections of '{}': {error}", resolved.entry.as_str()))
      })
    })
    .await
}
