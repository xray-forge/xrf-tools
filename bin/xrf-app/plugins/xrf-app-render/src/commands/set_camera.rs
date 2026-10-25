use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldCamera;

use crate::WorldState;

/// Describe a viewport's camera; described again from the same start, it keeps where it has been moved.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "set_camera")]
pub fn render_set_camera(world: State<'_, WorldState>, viewport: RenderViewportId, camera: WorldCamera) {
  world.lock().set_camera(viewport, camera);
}
