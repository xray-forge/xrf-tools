use std::sync::mpsc::Receiver;
use std::time::Duration;

use tauri::State;
use xrf_renderer::{RenderTextureReport, RenderViewportId};

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// How long a description waits for the render thread to take it up.
const DESCRIBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Say what became of every texture a viewport's level samples.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_textures"))]
#[tauri::command(rename = "describe_textures")]
pub async fn render_describe_textures(
  state: State<'_, RenderState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<RenderTextureReport>> {
  let answer: Receiver<Vec<RenderTextureReport>> = state.renderer.describe_textures(viewport);

  execution
    .run_blocking(
      "Describing a viewport's textures",
      move || -> TauriResult<Vec<RenderTextureReport>> {
        answer
          .recv_timeout(DESCRIBE_TIMEOUT)
          .map_err(|_| format!("Viewport {} answered no description", viewport.0))
      },
    )
    .await?
}
