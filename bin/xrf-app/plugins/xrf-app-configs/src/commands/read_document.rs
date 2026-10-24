use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;

use crate::request::ConfigsReadDocumentRequest;
use crate::{ConfigsDocument, ConfigsProject, ConfigsProjects};

/// Reads one config as the authored view renders it: its lines, and what only the parser knows about them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_document"))]
#[tauri::command(rename = "read_document")]
pub async fn configs_read_document(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsReadDocumentRequest,
) -> TauriResult<ConfigsDocument> {
  let ConfigsReadDocumentRequest { session_id, path } = request;

  // Taken before the hop, so the work does not hold the session against every other command. A snapshot outlives its
  // handle: closing the project mid-read leaves this answering what it was asked for, which is sound because reading
  // commits nothing back.
  let opened: Arc<SessionSnapshot<ConfigsProject>> = projects.require(session_id)?;

  // Bounded by the include tree of whichever entry point the file belongs to, which on a game tree is `system.ltx` and
  // most of the configs under it.
  execution
    .run_blocking("Configs document read", move || opened.read_document(&path))
    .await?
}
