use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::{ArchiveResolution, ArchiveSubjects};

/// Where the open subject looks for an engine path, in the order it looks.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_resolution"))]
#[tauri::command(rename = "describe_resolution")]
pub async fn archives_describe_resolution(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<SessionSnapshot<ArchiveResolution>> {
  log::info!("Describing archive resolution");

  let resolution: SessionSnapshot<ArchiveResolution> = subjects
    .read_on_pool(&execution, session_id, "Describing the archive resolution", |subject| {
      Ok(subject.describe_resolution())
    })
    .await?;

  log::info!(
    "Described {} searched source(s), {} unread",
    resolution.sources.len(),
    resolution.unread.len()
  );

  Ok(resolution)
}
