use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldLevelProblems;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::world_state::WorldState;

/// Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_problems"))]
#[tauri::command(rename = "describe_problems")]
pub async fn render_describe_problems(
  world: State<'_, WorldState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<WorldLevelProblems> {
  world
    .read(&execution, "Describing a level's problems", move |world| {
      world.describe_problems(viewport)
    })
    .await
}
