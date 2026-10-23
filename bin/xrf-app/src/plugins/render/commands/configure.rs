use tauri::State;
use xrf_renderer::RenderSettings;

use crate::plugins::render::renderer::Renderer;

/// Apply settings every native viewport draws with.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "configure"))]
#[tauri::command(rename = "configure")]
pub fn render_configure(renderer: State<'_, Renderer>, settings: RenderSettings) {
  renderer.configure(settings);
}
