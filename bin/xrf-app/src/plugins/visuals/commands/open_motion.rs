use std::sync::MutexGuard;

use tauri::State;
use xrf_visual::{VisualMotionBake, VisualMotionPositions};

use crate::core::assets::AssetMountState;
use crate::core::types::TauriResult;
use crate::plugins::visuals::pose::bake_named_motion;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// Pose the open visual through one of its motions, and report what came out.
///
/// Every frame is baked here and parked, so the `read_motion` that follows serves the same pose rather than composing
/// it again - the same split geometry uses, and for the same reason: a typed command cannot carry the bytes.
///
/// Baked whole rather than sampled per frame because playback runs at thirty frames a second. A measured motion
/// averages 78 frames, which for a fifty bone skeleton is tens of kilobytes: cheaper once than eighty times.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_motion"))]
#[tauri::command(rename = "open_motion")]
pub async fn visuals_open_motion(
  name: String,
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<VisualMotionBake> {
  log::info!("Posing motion: {name}");

  let mut selected: MutexGuard<Option<SelectedVisual>> = state
    .selected
    .lock()
    .map_err(|error| format!("Failed to pose motion - selection state is unavailable: {error}"))?;

  let Some(current) = selected.as_mut() else {
    return Err(String::from("Cannot pose a motion while no visual is open"));
  };

  let Some(skeleton) = current.skeleton.as_ref() else {
    return Err(String::from("The open visual carries no bind pose to animate"));
  };

  // Posed inside the roots the open used, so the animation files are looked for where the model's own references were.
  let posed: VisualMotionPositions =
    assets.with_probe(&current.roots, |probe| bake_named_motion(probe, skeleton, &current.dependencies, &name))??;

  let description: VisualMotionBake = posed.description.clone();

  current.posed = Some(posed);

  Ok(description)
}
