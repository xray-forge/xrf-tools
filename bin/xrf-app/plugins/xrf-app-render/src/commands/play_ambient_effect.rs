use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::WorldState;

/// Play a weather ambient effect near a viewport's camera at once, ending the one playing; none plays indoors.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "play_ambient_effect"))]
#[tauri::command(rename = "play_ambient_effect")]
pub fn render_play_ambient_effect(world: State<'_, WorldState>, viewport: RenderViewportId) {
  world.lock().play_ambient_effect(viewport);
}
