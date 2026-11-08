use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::WorldState;

/// Hear the scene a viewport shows from its camera: the focused window's active pane. Every other scene falls silent
/// and plays on.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "listen_viewport")]
pub(crate) fn render_listen_viewport(world: State<'_, WorldState>, viewport: RenderViewportId) {
  world.lock().set_listener(viewport);
}
