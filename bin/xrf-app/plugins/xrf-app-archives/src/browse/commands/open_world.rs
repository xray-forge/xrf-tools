use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_vfs::XrayRoots;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// Open a game folder as the engine mounts it: its archives and the loose tree standing in front of them. The world
/// follows commits: one that moves what its listing read lists it again, at the revision a page then refreshes to.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_world")]
pub async fn archives_open_world(
  session_id: SessionId,
  roots: XrayRoots,
  execution: State<'_, ExecutionState>,
  assets: State<'_, AssetMountState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<Arc<SessionSnapshot<ArchiveSubject>>> {
  log::info!("Opening archive world: {}", roots.describe());

  let listed: XrayRoots = roots.clone();
  let opened: Arc<SessionSnapshot<ArchiveSubject>> = subjects
    .open_followed(
      &execution,
      &assets,
      session_id,
      roots,
      "Opening the archive world",
      move |search| Ok(ArchiveSubject::read_world(search, listed)),
    )
    .await
    .map_err(|error| format!("Failed to open provided game folder: {error}"))?;

  if let ArchiveSubject::World { world, .. } = &**opened {
    log::info!(
      "Opened archive world: {} files, {} overridden, {} source(s)",
      world.files.len(),
      world.shadowed_count,
      world.mounts.len()
    );
  }

  Ok(opened)
}
