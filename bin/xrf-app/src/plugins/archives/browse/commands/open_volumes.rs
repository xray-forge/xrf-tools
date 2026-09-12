use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_archive::ArchiveProject;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// Open one archive volume, or every volume beneath a directory, as a single name table.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_volumes"))]
#[tauri::command(rename = "open_volumes")]
pub async fn archives_open_volumes(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  path: &str,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<Arc<SessionSnapshot<ArchiveSubject>>> {
  state.begin_open(session_id)?;
  log::info!("Opening archive volumes: {path}");

  let source: PathBuf = PathBuf::from(path);

  // Off the async worker: opening walks the directory and reads every volume's whole name table, which is work bounded
  // by the installation rather than by anything short enough for an IPC executor to hold.
  let project: ArchiveProject = execution
    .run_blocking("Opening the archive volumes", move || ArchiveProject::new(&source))
    .await?
    .map_err(|error| format!("Failed to open provided archive volumes: {error}"))?;

  log::info!("Opened {} archive volume(s)", project.archives.len());

  // Shared rather than copied: the frontend and the session hold the same index.
  state.commit_open(session_id, ArchiveSubject::Volumes { project })
}
