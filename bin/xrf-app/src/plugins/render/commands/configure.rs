use tauri::State;
use xrf_renderer::RenderSettings;

use crate::plugins::render::state::RenderState;

/// Apply settings every native viewport draws with.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "configure"))]
#[tauri::command(rename = "configure")]
pub fn render_configure(state: State<'_, RenderState>, settings: RenderSettings) {
  state.renderer.configure(settings);
}
