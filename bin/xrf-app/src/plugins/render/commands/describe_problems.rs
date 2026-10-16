use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldLevelProblems;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_problems"))]
#[tauri::command(rename = "describe_problems")]
pub async fn render_describe_problems(
  state: State<'_, RenderState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<WorldLevelProblems> {
  state
    .read_world(&execution, "Describing a level's problems", move |world| {
      world.describe_problems(viewport)
    })
    .await
}
