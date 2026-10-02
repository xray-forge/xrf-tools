use tauri::State;
use xrf_renderer::{RenderViewportId, RenderWeatherPlay, RenderWeatherTransition};

use crate::plugins::render::state::RenderState;

/// Play a weather in a viewport's level from now on: a cycle by name, a keyframe set by hand, or nothing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "play_weather"))]
#[tauri::command(rename = "play_weather")]
pub fn render_play_weather(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
  play: RenderWeatherPlay,
  transition: RenderWeatherTransition,
) {
  state.renderer.play_weather(viewport, play, transition);
}
