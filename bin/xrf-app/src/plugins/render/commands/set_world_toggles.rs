use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldToggles;

use crate::plugins::render::state::RenderState;

/// Set what of one viewport's world plays: the weather's rain, bolts and wind, the campfires, the ambient effects, and
/// which spawn groups stream in.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_world_toggles"))]
#[tauri::command(rename = "set_world_toggles")]
pub fn render_set_world_toggles(state: State<'_, RenderState>, viewport: RenderViewportId, toggles: WorldToggles) {
  state.lock_world().set_toggles(viewport, toggles);
}
