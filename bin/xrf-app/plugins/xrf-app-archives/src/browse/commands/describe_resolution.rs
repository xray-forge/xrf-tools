use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::{ArchiveResolution, ArchiveSubject, ArchiveSubjects};

/// Where the open subject looks for an engine path, in the order it looks.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_resolution"))]
#[tauri::command(rename = "describe_resolution")]
pub async fn archives_describe_resolution(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<ArchiveResolution> {
  log::info!("Describing archive resolution");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  let resolution: ArchiveResolution = execution
    .run_blocking("Describing the archive resolution", move || {
      subject.describe_resolution()
    })
    .await?;

  log::info!(
    "Described {} searched source(s), {} unread",
    resolution.sources.len(),
    resolution.unread.len()
  );

  Ok(resolution)
}
