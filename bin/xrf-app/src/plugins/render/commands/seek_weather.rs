use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::plugins::render::state::RenderState;

/// Play a viewport's weather on from a time of day, in seconds since midnight, ending the effect playing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "seek_weather"))]
#[tauri::command(rename = "seek_weather")]
pub fn render_seek_weather(state: State<'_, RenderState>, viewport: RenderViewportId, time: f32) {
  state.lock_world().seek_weather(viewport, time);
}
