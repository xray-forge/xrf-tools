use std::sync::Arc;

use tauri::State;
use xrf_archive::{ArchiveProject, ArchiveSharedPayload};

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// Payloads that several entries of the open volume set locate at once.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_shared_payloads"))]
#[tauri::command(rename = "list_shared_payloads")]
pub async fn archives_list_shared_payloads(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<Vec<ArchiveSharedPayload>> {
  log::info!("Listing archive shared payloads");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;

  // Off the async worker: one pass over the merged name table, which an installation sizes rather than a gesture.
  let payloads: Vec<ArchiveSharedPayload> = execution
    .run_blocking("Listing the archive shared payloads", move || {
      subject.require_volumes().map(ArchiveProject::list_shared_payloads)
    })
    .await??;

  log::info!("Listed {} shared archive payloads", payloads.len());

  Ok(payloads)
}
