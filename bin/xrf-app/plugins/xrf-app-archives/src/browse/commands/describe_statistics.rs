use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_archive_stats::ArchiveStatistics;

use crate::browse::ArchiveSubjects;

/// What the open subject holds, broken down by extension, folder, size, and where its files come from.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_statistics"))]
#[tauri::command(rename = "describe_statistics")]
pub async fn archives_describe_statistics(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<SessionSnapshot<ArchiveStatistics>> {
  log::info!("Describing archive statistics");

  // Off the async worker: one pass over the entries, which an installation sizes rather than a gesture.
  let statistics: SessionSnapshot<ArchiveStatistics> = subjects
    .read_on_pool(&execution, session_id, "Describing the archive statistics", |subject| {
      Ok(subject.describe_statistics())
    })
    .await?;

  log::info!(
    "Described {} files across {} extensions",
    statistics.overview.total.files,
    statistics.extensions.len()
  );

  Ok(statistics)
}
