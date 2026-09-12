use std::sync::Arc;

use tauri::State;
use xrf_vfs::XrayPathCollision;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// Entries the open subject holds that no engine lookup can reach.
///
/// Answered on demand rather than stored beside the subject, so there is one source of truth and no second slot a
/// close could leave stale. Asking the mount layer for it is what keeps the explorer's answer the same one
/// `gamedata list` and `archive verify` give.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_collisions"))]
#[tauri::command(rename = "list_collisions")]
pub async fn archives_list_collisions(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<Vec<XrayPathCollision>> {
  log::info!("Listing unreachable entries");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;

  // Off the async worker: a volume set folds its merged name table here, which an installation sizes rather than a
  // gesture.
  let collisions: Vec<XrayPathCollision> = execution
    .run_blocking("Listing unreachable entries", move || subject.list_collisions())
    .await?;

  log::info!("Listed {} unreachable entries", collisions.len());

  Ok(collisions)
}
