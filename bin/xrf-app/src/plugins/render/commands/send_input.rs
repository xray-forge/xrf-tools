use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldInputEvent;

use crate::plugins::render::world_state::WorldState;

/// Hand a viewport one gesture the page heard over it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "send_input"))]
#[tauri::command(rename = "send_input")]
pub fn render_send_input(world: State<'_, WorldState>, viewport: RenderViewportId, event: WorldInputEvent) {
  world.lock().send_input(viewport, &event);
}
