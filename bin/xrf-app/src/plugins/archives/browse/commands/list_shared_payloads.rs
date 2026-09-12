use std::sync::Arc;

use tauri::State;
use xrf_archive::{ArchiveProject, ArchiveSharedPayload};

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// Payloads that several entries of the open volume set locate at once.
///
/// A volume set only: the group is derived from equal name-table descriptors, which a mounted world does not keep, so
/// answering for one would mean answering a question it cannot see. The refusal is [`ArchiveSubject::require_volumes`]
/// rather than an empty list, because nothing shared is a different claim from nothing knowable.
///
/// Derived on demand out of the open subject rather than stored beside it, the way `list_collisions` answers, so a
/// close cannot leave a stale answer behind. The derivation is `xrf-archive`'s: the format keeps no alias field, so
/// this is what a reader observes from equal descriptors and never what the packer recorded. See
/// [`ArchiveSharedPayload`].
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
