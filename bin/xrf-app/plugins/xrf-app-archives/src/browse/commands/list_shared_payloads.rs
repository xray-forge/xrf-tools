use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_archive::{ArchiveProject, ArchiveSharedPayload};

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// Payloads that several entries of the open volume set locate at once.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_shared_payloads"))]
#[tauri::command(rename = "list_shared_payloads")]
pub async fn archives_list_shared_payloads(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<Vec<ArchiveSharedPayload>> {
  log::info!("Listing archive shared payloads");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  // Off the async worker: one pass over the merged name table, which an installation sizes rather than a gesture.
  let payloads: Vec<ArchiveSharedPayload> = execution
    .run_blocking("Listing the archive shared payloads", move || {
      subject.require_volumes().map(ArchiveProject::list_shared_payloads)
    })
    .await??;

  log::info!("Listed {} shared archive payloads", payloads.len());

  Ok(payloads)
}
