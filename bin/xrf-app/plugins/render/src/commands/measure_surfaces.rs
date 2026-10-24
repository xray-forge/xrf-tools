use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::types::TauriResult;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldSurfaceGeometry;

use crate::world_state::WorldState;

/// Count what each shader table entry of a viewport's level draws across the sectors resident.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "measure_surfaces"))]
#[tauri::command(rename = "measure_surfaces")]
pub async fn render_measure_surfaces(
  world: State<'_, WorldState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<WorldSurfaceGeometry>> {
  world
    .read(&execution, "Measuring a level's surfaces", move |world| {
      world.measure_surfaces(viewport)
    })
    .await
}
