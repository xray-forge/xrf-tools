use std::sync::Arc;

use tauri::State;
use xrf_visual::{VisualMotionBake, VisualMotionPose};

use crate::core::assets::AssetMountState;
use crate::core::session::{DocumentSessionId, DocumentSnapshot};
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
  session_id: DocumentSessionId,
  motion_id: DocumentSessionId,
  name: String,
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<DocumentSnapshot<VisualMotionBake>> {
  log::info!("Posing motion: {name}");

  let current: Arc<DocumentSnapshot<SelectedVisual>> = state.selected.require(session_id)?;

  current.posed.begin_open(motion_id)?;

  let Some(skeleton) = current.skeleton.as_ref() else {
    return Err(String::from("The open visual carries no bind pose to animate"));
  };

  // Posed inside the roots the open used, so the animation files are looked for where the model's own references were.
  let posed: VisualMotionPose = assets.with_probe(&current.roots, |probe| {
    bake_named_motion(probe, skeleton, &current.dependencies, &name)
  })??;

  let opened: Arc<DocumentSnapshot<VisualMotionPose>> = current.posed.commit_open(motion_id, posed)?;

  Ok(opened.map(|pose| pose.description.clone()))
}
