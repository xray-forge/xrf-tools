use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldCameraCommand;

use crate::WorldState;

/// Ask a viewport's camera to reset or dolly.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "command_camera"))]
#[tauri::command(rename = "command_camera")]
pub fn render_command_camera(world: State<'_, WorldState>, viewport: RenderViewportId, command: WorldCameraCommand) {
  world.lock().command_camera(viewport, command);
}
