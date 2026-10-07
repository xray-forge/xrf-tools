use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldSurfaceGeometry;

use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// Count what each shader table entry of a viewport's level draws across the sectors resident.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "measure_surfaces"))]
#[tauri::command(rename = "measure_surfaces")]
pub async fn render_measure_surfaces(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<WorldSurfaceGeometry>> {
  Ok(state.lock_world().measure_surfaces(viewport))
}
