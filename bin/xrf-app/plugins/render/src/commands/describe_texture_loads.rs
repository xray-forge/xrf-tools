use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::types::TauriResult;
use xrf_renderer::{RenderTextureReport, RenderViewportId};

use crate::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::renderer::Renderer;

/// Say what became of every texture a viewport's level samples.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_texture_loads"))]
#[tauri::command(rename = "describe_texture_loads")]
pub async fn render_describe_texture_loads(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<RenderTextureReport>> {
  let answer: Receiver<Vec<RenderTextureReport>> = renderer.describe_texture_loads(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no description", viewport.0),
  )
  .await
}
