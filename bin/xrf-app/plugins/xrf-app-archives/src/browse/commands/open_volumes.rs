use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_archive::ArchiveProject;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// Open one archive volume, or every volume beneath a directory, as a single name table.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_volumes")]
pub async fn archives_open_volumes(
  session_id: SessionId,
  path: PathBuf,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<Arc<SessionSnapshot<ArchiveSubject>>> {
  log::info!("Opening archive volumes: {}", path.display());

  let opened: Arc<SessionSnapshot<ArchiveSubject>> = subjects
    .open_on_pool(&execution, session_id, "Opening the archive volumes", move || {
      ArchiveProject::new(&path)
        .map(ArchiveSubject::of_volumes)
        .map_err(|error| format!("Failed to open provided archive volumes: {error}"))
    })
    .await?;

  if let ArchiveSubject::Volumes { project, .. } = &**opened {
    log::info!("Opened {} archive volume(s)", project.archives.len());
  }

  Ok(opened)
}
