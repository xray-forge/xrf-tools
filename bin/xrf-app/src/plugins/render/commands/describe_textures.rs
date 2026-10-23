use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_renderer::{RenderTextureReport, RenderViewportId};

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::renderer::Renderer;

/// Say what became of every texture a viewport's level samples.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_textures"))]
#[tauri::command(rename = "describe_textures")]
pub async fn render_describe_textures(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<RenderTextureReport>> {
  let answer: Receiver<Vec<RenderTextureReport>> = renderer.describe_textures(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no description", viewport.0),
  )
  .await
}
