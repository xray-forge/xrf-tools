use std::sync::Arc;

use tauri::State;
use xrf_archive::{ArchiveProject, ArchiveSharedPayload};

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

/// Payloads that several entries of the open volume set locate at once.
///
/// Derived on demand out of the open project rather than stored beside it, the way `list_collisions` answers, so a
/// close cannot leave a stale answer behind. The derivation is `xrf-archive`'s: the format keeps no alias field, so
/// this is what a reader observes from equal descriptors and never what the packer recorded. See
/// [`ArchiveSharedPayload`].
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_shared_payloads"))]
#[tauri::command(rename = "list_shared_payloads")]
pub async fn archives_list_shared_payloads(
  execution: State<'_, ExecutionState>,
  state: State<'_, ArchiveProjectState>,
) -> TauriResult<Vec<ArchiveSharedPayload>> {
  log::info!("Listing archive project shared payloads");

  let project: Arc<ArchiveProject> = state.require("list shared payloads")?;

  // Off the async worker: one pass over the merged name table, which an installation sizes rather than a gesture.
  let payloads: Vec<ArchiveSharedPayload> = execution
    .run_blocking("Listing the archive project shared payloads", move || {
      project.list_shared_payloads()
    })
    .await?;

  log::info!("Listed {} shared archive payloads", payloads.len());

  Ok(payloads)
}
