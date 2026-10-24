use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldToggles;

use crate::world_state::WorldState;

/// Set what of one viewport's world plays: the weather's rain, bolts and wind, the campfires, the ambient effects, and
/// which spawn groups stream in.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_world_toggles"))]
#[tauri::command(rename = "set_world_toggles")]
pub fn render_set_world_toggles(world: State<'_, WorldState>, viewport: RenderViewportId, toggles: WorldToggles) {
  world.lock().set_toggles(viewport, toggles);
}
