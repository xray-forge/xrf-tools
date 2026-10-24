use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_archive_stats::ArchiveStatistics;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// What the open subject holds, broken down by extension, folder, size, and where its files come from.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_statistics"))]
#[tauri::command(rename = "describe_statistics")]
pub async fn archives_describe_statistics(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<ArchiveStatistics> {
  log::info!("Describing archive statistics");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  // Off the async worker: one pass over the entries, which an installation sizes rather than a gesture.
  let statistics: ArchiveStatistics = execution
    .run_blocking("Describing the archive statistics", move || {
      subject.describe_statistics()
    })
    .await?;

  log::info!(
    "Described {} files across {} extensions",
    statistics.overview.total.files,
    statistics.extensions.len()
  );

  Ok(statistics)
}
