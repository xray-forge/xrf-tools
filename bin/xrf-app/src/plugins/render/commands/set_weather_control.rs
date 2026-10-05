use tauri::State;
use xrf_renderer::{RenderViewportId, RenderWeatherControl};

use crate::plugins::render::state::RenderState;

/// Set how a viewport's weather clock runs.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_weather_control"))]
#[tauri::command(rename = "set_weather_control")]
pub fn render_set_weather_control(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
  control: RenderWeatherControl,
) {
  state.lock_world().set_weather_control(viewport, control);
}
