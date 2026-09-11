use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::visuals::pose::list_motion_names;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// Every motion the open visual can play, by name.
///
/// Asked for rather than returned by `open_model`, because naming them means reading each animation file the visual
/// references - about fifty milliseconds each against a seventy millisecond open. The viewer already knows whether a
/// visual animates at all, from its references, so nothing needs this until something is about to play one.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_motions"))]
#[tauri::command(rename = "list_motions")]
pub async fn visuals_list_motions(
  session_id: DocumentSessionId,
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<Vec<String>> {
  let current: Arc<DocumentSnapshot<SelectedVisual>> = state.selected.require(session_id)?;

  let Some(skeleton) = current.skeleton.as_ref() else {
    // No bind pose means nothing to pose, so the names would be unplayable even where they exist.
    return Ok(Vec::new());
  };

  let names: Vec<String> = assets.with_probe(&current.roots, |probe| {
    list_motion_names(probe, skeleton, &current.dependencies)
  })?;

  log::info!("Listed {} motions for: {}", names.len(), current.source.label());

  Ok(names)
}
