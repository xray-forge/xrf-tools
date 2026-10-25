use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldLevelProblems;

use crate::WorldState;

/// Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
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
