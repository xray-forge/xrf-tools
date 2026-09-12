use std::sync::Arc;

use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject, ArchiveWorld};

/// Open a game folder as the engine mounts it: its archives and the loose tree standing in front of them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_world"))]
#[tauri::command(rename = "open_world")]
pub async fn archives_open_world(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  roots: XrayRoots,
  assets: State<'_, AssetMountState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<Arc<SessionSnapshot<ArchiveSubject>>> {
  state.begin_open(session_id)?;
  log::info!("Opening archive world: {}", roots.describe());

  let assets: AssetMountState = AssetMountState::clone(&assets);

  // Off the async worker: mounting indexes every volume's name table and walks every loose tree, then the listing
  // measures each winner - work bounded by the installation rather than by anything an IPC executor should hold.
  let world: ArchiveWorld = execution
    .run_blocking("Opening the archive world", move || {
      assets.with_probe(&roots, |probe| ArchiveWorld::list(probe, roots.clone()))
    })
    .await?
    .map_err(|error| format!("Failed to open provided game folder: {error}"))?;

  log::info!(
    "Opened archive world: {} files, {} overridden, {} source(s)",
    world.files.len(),
    world.shadowed_count,
    world.mounts.len()
  );

  state.commit_open(session_id, ArchiveSubject::World { world })
}
