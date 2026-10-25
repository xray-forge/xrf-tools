use tauri::State;
use xrf_renderer::RenderLibraries;

use crate::Renderer;

/// Report the versions of the graphics libraries the renderer ships with; currently only the DLSS runtime.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_libraries")]
pub fn render_describe_libraries(renderer: State<'_, Renderer>) -> RenderLibraries {
  renderer.get_queue().describe_libraries()
}
