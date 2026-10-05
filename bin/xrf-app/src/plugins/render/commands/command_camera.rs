use tauri::State;
use xrf_renderer::{RenderCameraCommand, RenderViewportId};

use crate::plugins::render::state::RenderState;

/// Ask a viewport's camera to reset or dolly.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "command_camera"))]
#[tauri::command(rename = "command_camera")]
pub fn render_command_camera(state: State<'_, RenderState>, viewport: RenderViewportId, command: RenderCameraCommand) {
  state.lock_world().command_camera(viewport, command);
}
