use std::sync::MutexGuard;

use tauri::State;

use crate::core::assets::AssetMountState;
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
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<Vec<String>> {
  let selected: MutexGuard<Option<SelectedVisual>> = state
    .selected
    .lock()
    .map_err(|error| format!("Failed to list motions - selection state is unavailable: {error}"))?;

  let Some(current) = selected.as_ref() else {
    return Ok(Vec::new());
  };

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
