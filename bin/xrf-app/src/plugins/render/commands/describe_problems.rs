use std::sync::mpsc::Receiver;
use std::time::Duration;

use tauri::State;
use xrf_renderer::{RenderLevelProblems, RenderViewportId};

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// How long a description waits for the render thread to take it up.
const DESCRIBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_problems"))]
#[tauri::command(rename = "describe_problems")]
pub async fn render_describe_problems(
  state: State<'_, RenderState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<RenderLevelProblems> {
  let answer: Receiver<RenderLevelProblems> = state.renderer.describe_problems(viewport);

  execution
    .run_blocking(
      "Describing a viewport's problems",
      move || -> TauriResult<RenderLevelProblems> {
        answer
          .recv_timeout(DESCRIBE_TIMEOUT)
          .map_err(|_| format!("Viewport {} answered no description", viewport.0))
      },
    )
    .await?
}
