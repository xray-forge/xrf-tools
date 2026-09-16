use std::sync::Arc;

use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveOverrideReport, ArchiveSubject};

/// What the open subject answers with more than one copy, and what it cannot reach at all.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_overrides"))]
#[tauri::command(rename = "list_overrides")]
pub async fn archives_list_overrides(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<ArchiveOverrideReport> {
  log::info!("Describing archive overrides");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;

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
