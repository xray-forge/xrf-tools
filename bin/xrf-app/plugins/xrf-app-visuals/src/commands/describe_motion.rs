use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_visual::VisualMotionBake;

use crate::{SelectedVisual, VisualModels};

/// Describe one of the open visual's motions as a pose of it would hold it, its frames, bones and length, posing none;
/// read on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_motion"))]
#[tauri::command(rename = "describe_motion")]
pub async fn visuals_describe_motion(
  session_id: SessionId,
  name: String,
  models: State<'_, VisualModels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<VisualMotionBake> {
  let current: Arc<SessionSnapshot<SelectedVisual>> = models.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Describing a visual's motion", move || {
      let Some(motions) = current.motions.as_ref() else {
        return Err(String::from("The open visual carries no bind pose to animate"));
      };

      // Read inside the roots the open used, so the animation files are looked for where the model's own references
      // were.
      motions
        .describe_named(
          &assets.search(&current.description.roots)?,
          &current.description.dependencies,
          &name,
        )
        .map_err(|error| error.to_string())
    })
    .await?
}
