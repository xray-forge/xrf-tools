use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_archive::{ArchiveProject, ArchiveSharedPayload};

use crate::browse::ArchiveSubjects;

/// Payloads that several entries of the open volume set locate at once.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "list_shared_payloads")]
pub async fn archives_list_shared_payloads(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<SessionSnapshot<Vec<ArchiveSharedPayload>>> {
  log::info!("Listing archive shared payloads");

  // Off the async worker: one pass over the merged name table, which an installation sizes rather than a gesture.
  let payloads: SessionSnapshot<Vec<ArchiveSharedPayload>> = subjects
    .read_on_pool(
      &execution,
      session_id,
      "Listing the archive shared payloads",
      |subject| subject.require_volumes().map(ArchiveProject::list_shared_payloads),
    )
    .await?;

  log::info!("Listed {} shared archive payloads", payloads.len());

  Ok(payloads)
}
