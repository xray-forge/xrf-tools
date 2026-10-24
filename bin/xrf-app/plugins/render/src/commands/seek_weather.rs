use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::world_state::WorldState;

/// Play a viewport's weather on from a time of day, in seconds since midnight, ending the effect playing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "seek_weather"))]
#[tauri::command(rename = "seek_weather")]
pub fn render_seek_weather(world: State<'_, WorldState>, viewport: RenderViewportId, time: f32) {
  world.lock().seek_weather(viewport, time);
}
