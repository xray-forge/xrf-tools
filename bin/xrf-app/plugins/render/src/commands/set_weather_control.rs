use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldWeatherControl;

use crate::world_state::WorldState;

/// Set how a viewport's weather clock runs.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_weather_control"))]
#[tauri::command(rename = "set_weather_control")]
pub fn render_set_weather_control(
  world: State<'_, WorldState>,
  viewport: RenderViewportId,
  control: WorldWeatherControl,
) {
  world.lock().set_weather_control(viewport, control);
}
