use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::plugins::render::state::RenderState;

/// Play a weather effect over a viewport's cycle from its clock's time, or end the one playing for none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "play_weather_effect"))]
#[tauri::command(rename = "play_weather_effect")]
pub fn render_play_weather_effect(state: State<'_, RenderState>, viewport: RenderViewportId, name: Option<String>) {
  state.renderer.play_weather_effect(viewport, name);
}
