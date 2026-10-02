use tauri::State;
use xrf_renderer::{RenderCamera, RenderViewportId};

use crate::plugins::render::state::RenderState;

/// Describe a viewport's camera; described again from the same start, it keeps where it has been moved.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_camera"))]
#[tauri::command(rename = "set_camera")]
pub fn render_set_camera(state: State<'_, RenderState>, viewport: RenderViewportId, camera: RenderCamera) {
  state.renderer.set_camera(viewport, camera);
}
