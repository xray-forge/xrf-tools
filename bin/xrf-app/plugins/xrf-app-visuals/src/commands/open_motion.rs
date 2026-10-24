use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_visual::{VisualMotionBake, VisualMotionPose};

use crate::posed_visual_motion::PosedVisualMotion;
use crate::selected_visual::SelectedVisual;
use crate::state::VisualState;

/// Pose the open visual through one of its motions, held as `motion_id`, an opening of the viewer's own, and report
/// what came out.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_motion"))]
#[tauri::command(rename = "open_motion")]
pub async fn visuals_open_motion(
  session_id: SessionId,
  motion_id: SessionId,
  name: String,
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<SessionSnapshot<VisualMotionBake>> {
  log::info!("Posing motion: {name}");

  let current: Arc<SessionSnapshot<SelectedVisual>> = state.models.require(session_id)?;
  // Refused before the pose is reserved, so a model with nothing to animate leaves no opening behind.
  let Some(motions) = current.motions.as_ref() else {
    return Err(String::from("The open visual carries no bind pose to animate"));
  };

  state.poses.begin_open(motion_id)?;

  // Posed inside the roots the open used, so the animation files are looked for where the model's own references were.
  let baked: TauriResult<VisualMotionPose> = assets.search(&current.roots).and_then(|search| {
    motions
      .bake_named(&search, &current.dependencies, &name)
      .map_err(|error| error.to_string())
  });
  // A pose that does not bake leaves no opening behind: the page forgets one whose open failed.
  let pose: VisualMotionPose = baked.inspect_err(|_| {
    if let Err(error) = state.poses.close(&[motion_id]) {
      log::error!("A pose that did not bake could not be let go: {error}");
    }
  })?;
  let opened: Arc<SessionSnapshot<PosedVisualMotion>> = state.poses.commit_open(
    motion_id,
    PosedVisualMotion {
      model: session_id,
      pose,
    },
  )?;

  Ok(opened.map(|posed| posed.pose.description.clone()))
}
