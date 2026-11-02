use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::WorldState;

/// Play a weather effect over a viewport's cycle from its clock's time, or end the one playing for none.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "play_weather_effect")]
pub fn render_play_weather_effect(world: State<'_, WorldState>, viewport: RenderViewportId, name: Option<String>) {
  world.lock().play_weather_effect(viewport, name.as_deref());
}
