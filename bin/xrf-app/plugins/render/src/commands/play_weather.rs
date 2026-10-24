use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::{WorldWeatherPlay, WorldWeatherTransition};

use crate::world_state::WorldState;

/// Play a weather in a viewport's level from now on: a cycle by name, a keyframe set by hand, or nothing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "play_weather"))]
#[tauri::command(rename = "play_weather")]
pub fn render_play_weather(
  world: State<'_, WorldState>,
  viewport: RenderViewportId,
  play: WorldWeatherPlay,
  transition: WorldWeatherTransition,
) {
  world.lock().play_weather(viewport, play, transition);
}
