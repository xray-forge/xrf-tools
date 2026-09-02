use std::sync::Arc;

use tauri::State;
use xrf_archive::ArchiveProject;
use xrf_vfs::{XrayArchiveSource, XrayPathCollision};

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

/// Entries the open volume set holds that no engine lookup can reach.
///
/// Answered on demand out of the open project rather than stored beside it, so there is one source of truth and no
/// second slot to keep in step with an open or a close.
///
/// Not part of [`ArchiveProject`]: the project keys entries by the name their volume's header authored, and folding
/// those onto engine identities is `xrf-vfs`'s to do. Asking the mount layer here is what keeps the explorer's answer
/// the same one `gamedata list` and `archive verify` give.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_collisions"))]
#[tauri::command(rename = "list_collisions")]
pub async fn archives_list_collisions(
  execution: State<'_, ExecutionState>,
  state: State<'_, ArchiveProjectState>,
) -> TauriResult<Vec<XrayPathCollision>> {
  log::info!("Listing archive project collisions");

  let project: Arc<ArchiveProject> = state.require("list collisions")?;

  // Off the async worker: the fold walks the merged name table, which an installation sizes rather than a gesture.
  let collisions: Vec<XrayPathCollision> = execution
    .run_blocking("Listing the archive project collisions", move || {
      XrayArchiveSource::list_collisions_of(&project)
    })
    .await?;

  log::info!("Listed {} unreachable archive entries", collisions.len());

  Ok(collisions)
}
