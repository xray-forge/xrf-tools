use std::sync::Arc;

use tauri::State;
use xrf_archive::{ArchiveProject, ProjectReadResult};

use crate::core::session::SessionId;
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_file"))]
#[tauri::command(rename = "read_file")]
pub async fn archives_read_file(
  session_id: SessionId,
  path: &str,
  state: State<'_, ArchiveProjectState>,
) -> TauriResult<ProjectReadResult> {
  log::info!("Reading archive file: {}", path);

  let project: Arc<SessionSnapshot<ArchiveProject>> = state.require(session_id)?;

  project.read_file_as_string(path).map_err(|error| error.to_string())
}
