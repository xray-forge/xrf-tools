use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::{ArchiveOverrideReport, ArchiveSubjects};

/// What the open subject answers with more than one copy, and what it cannot reach at all.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_overrides"))]
#[tauri::command(rename = "list_overrides")]
pub async fn archives_list_overrides(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<SessionSnapshot<ArchiveOverrideReport>> {
  log::info!("Describing archive overrides");

  let report: SessionSnapshot<ArchiveOverrideReport> = subjects
    .read_on_pool(&execution, session_id, "Describing the archive overrides", |subject| {
      Ok(subject.describe_overrides())
    })
    .await?;

  log::info!(
    "Described {} overridden and {} unreachable entries",
    report.overridden.len(),
    report.unreachable.len()
  );

  Ok(report)
}
