use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_visual::{VisualMotionBake, VisualMotionPose};

use crate::{PosedVisualMotion, SelectedVisual, VisualModels, VisualPoses};

/// Pose the open visual through one of its motions, held as `motion_id`, an opening of the viewer's own, and report
/// what came out; baked on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_motion"))]
#[tauri::command(rename = "open_motion")]
pub async fn visuals_open_motion(
  session_id: SessionId,
  motion_id: SessionId,
  name: String,
  models: State<'_, VisualModels>,
  poses: State<'_, VisualPoses>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<VisualMotionBake>> {
  log::info!("Posing motion: {name}");

  let current: Arc<SessionSnapshot<SelectedVisual>> = models.require(session_id)?;

  // Refused before the pose is reserved, so a model with nothing to animate leaves no opening behind.
  if current.motions.is_none() {
    return Err(String::from("The open visual carries no bind pose to animate"));
  }

  let assets: AssetMountState = AssetMountState::clone(&assets);
  // A pose that does not bake gives its opening up, so the page, which forgets an open that failed, leaves nothing
  // behind.
  let opened: Arc<SessionSnapshot<PosedVisualMotion>> = poses
    .open_on_pool(&execution, motion_id, "Posing a visual's motion", move || {
      // Posed inside the roots the open used, so the animation files are looked for where the model's own references
      // were.
      let pose: VisualMotionPose = current
        .motions
        .as_ref()
        .ok_or_else(|| String::from("The open visual carries no bind pose to animate"))?
        .bake_named(
          &assets.search(&current.description.roots)?,
          &current.description.dependencies,
          &name,
        )
        .map_err(|error| error.to_string())?;

      Ok(PosedVisualMotion {
        model: session_id,
        pose: Arc::new(pose),
      })
    })
    .await?;

  Ok(opened.map(|posed| posed.pose.description.clone()))
}
