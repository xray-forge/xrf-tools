use std::sync::Arc;

use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::archive_world_entry::ArchiveWorldEntry;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// Every engine path the open subject answers with more than one copy, winner first.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_overrides"))]
#[tauri::command(rename = "list_overrides")]
pub async fn archives_list_overrides(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<Vec<ArchiveWorldEntry>> {
  log::info!("Listing overridden entries");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;

  let overrides: Vec<ArchiveWorldEntry> = execution
    .run_blocking("Listing overridden entries", move || subject.list_overrides())
    .await?;

  log::info!("Listed {} overridden entries", overrides.len());

  Ok(overrides)
}
