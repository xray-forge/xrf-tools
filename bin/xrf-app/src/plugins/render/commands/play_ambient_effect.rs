use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::plugins::render::state::RenderState;

/// Play a weather ambient effect near a viewport's camera at once, ending the one playing; none plays indoors.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "play_ambient_effect"))]
#[tauri::command(rename = "play_ambient_effect")]
pub fn render_play_ambient_effect(state: State<'_, RenderState>, viewport: RenderViewportId) {
  state.renderer.play_ambient_effect(viewport);
}
