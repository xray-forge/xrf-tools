use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveResolution, ArchiveSubject};

/// Where the open subject looks for an engine path, in the order it looks.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_resolution"))]
#[tauri::command(rename = "describe_resolution")]
pub async fn archives_describe_resolution(
  session_id: SessionId,
  execution: State<'_, ExecutionState>,
  assets: State<'_, AssetMountState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<ArchiveResolution> {
  log::info!("Describing archive resolution");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  let resolution: ArchiveResolution = execution
    .run_blocking("Describing the archive resolution", move || {
      subject.describe_resolution(&assets)
    })
    .await??;

  log::info!(
    "Described {} searched source(s), {} unread",
    resolution.sources.len(),
    resolution.unread.len()
  );

  Ok(resolution)
}
