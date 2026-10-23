use std::sync::Arc;

use tauri::State;
use xrf_visual::VisualMotionBake;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::visuals::pose::describe_named_motion;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// Describe one of the open visual's motions as a pose of it would hold it, its frames, bones and length, posing none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_motion"))]
#[tauri::command(rename = "describe_motion")]
pub async fn visuals_describe_motion(
  session_id: SessionId,
  name: String,
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<VisualMotionBake> {
  let current: Arc<SessionSnapshot<SelectedVisual>> = state.selected.require(session_id)?;

  let Some(skeleton) = current.skeleton.as_ref() else {
    return Err(String::from("The open visual carries no bind pose to animate"));
  };

  // Read inside the roots the open used, so the animation files are looked for where the model's own references were.
  describe_named_motion(&assets.search(&current.roots)?, skeleton, &current.dependencies, &name)
}
