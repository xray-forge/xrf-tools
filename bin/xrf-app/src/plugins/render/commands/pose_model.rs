use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldModelPose;

use crate::plugins::render::world_state::WorldState;

/// Stand one viewport's skinned models in a pose.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pose_model"))]
#[tauri::command(rename = "pose_model")]
pub fn render_pose_model(world: State<'_, WorldState>, viewport: RenderViewportId, pose: WorldModelPose) {
  world.lock().pose_model(viewport, &pose);
}
