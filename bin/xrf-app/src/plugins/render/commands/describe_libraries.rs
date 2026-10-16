use tauri::State;
use xrf_renderer::RenderLibraries;

use crate::plugins::render::state::RenderState;

/// Report the versions of the graphics libraries the renderer ships with; currently only the DLSS runtime.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_libraries"))]
#[tauri::command(rename = "describe_libraries")]
pub fn render_describe_libraries(state: State<'_, RenderState>) -> RenderLibraries {
  state.renderer.get_queue().describe_libraries()
}
