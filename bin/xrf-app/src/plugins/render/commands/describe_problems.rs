use tauri::State;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldLevelProblems;

use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_problems"))]
#[tauri::command(rename = "describe_problems")]
pub async fn render_describe_problems(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
) -> TauriResult<WorldLevelProblems> {
  Ok(state.lock_world().describe_problems(viewport))
}
