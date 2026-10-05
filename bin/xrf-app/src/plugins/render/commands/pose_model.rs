use tauri::State;
use xrf_renderer::{RenderModelPose, RenderViewportId};

use crate::plugins::render::state::RenderState;

/// Stand one viewport's skinned models in a pose.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pose_model"))]
#[tauri::command(rename = "pose_model")]
pub fn render_pose_model(state: State<'_, RenderState>, viewport: RenderViewportId, pose: RenderModelPose) {
  state.lock_world().pose_model(viewport, &pose);
}
