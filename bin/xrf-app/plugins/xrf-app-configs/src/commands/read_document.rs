use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_utils::error_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::project::{ConfigsDocument, ConfigsProjects};
use crate::request::ConfigsReadDocumentRequest;

/// Reads one config as the authored view renders it: its lines, and what only the parser knows about them.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "read_document")]
pub async fn configs_read_document(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  request: ConfigsReadDocumentRequest,
) -> TauriResult<SessionSnapshot<ConfigsDocument>> {
  let ConfigsReadDocumentRequest { session_id, path } = request;
  let path: XrayLogicalPath = XrayLogicalPath::new(&path).map_err(error_to_string)?;

  // Bounded by the include tree of whichever entry point the file belongs to, which on a game tree is `system.ltx` and
  // most of the configs under it. Closing the project mid-read leaves this answering what it was asked for, which is
  // sound because reading commits nothing back.
  projects
    .read_on_pool(&execution, session_id, "Configs document read", move |opened| {
      opened.read_document(&path)
    })
    .await
}
