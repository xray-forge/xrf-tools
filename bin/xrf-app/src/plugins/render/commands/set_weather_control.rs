use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldWeatherControl;

use crate::plugins::render::state::RenderState;

/// Set how a viewport's weather clock runs.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_weather_control"))]
#[tauri::command(rename = "set_weather_control")]
pub fn render_set_weather_control(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
  control: WorldWeatherControl,
) {
  state.lock_world().set_weather_control(viewport, control);
}
