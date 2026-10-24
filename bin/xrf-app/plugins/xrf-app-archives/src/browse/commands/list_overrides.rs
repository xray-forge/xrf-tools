use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::{ArchiveOverrideReport, ArchiveSubject, ArchiveSubjects};

/// What the open subject answers with more than one copy, and what it cannot reach at all.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_overrides"))]
#[tauri::command(rename = "list_overrides")]
pub async fn archives_list_overrides(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<ArchiveOverrideReport> {
  log::info!("Describing archive overrides");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  let report: ArchiveOverrideReport = execution
    .run_blocking("Describing the archive overrides", move || subject.describe_overrides())
    .await?;

  log::info!(
    "Described {} overridden and {} unreachable entries",
    report.overridden.len(),
    report.unreachable.len()
  );

  Ok(report)
}
