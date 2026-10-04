use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_renderer::{RenderTextureReport, RenderViewportId};

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::state::RenderState;

/// Say what became of every texture a viewport's level samples.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_textures"))]
#[tauri::command(rename = "describe_textures")]
pub async fn render_describe_textures(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<RenderTextureReport>> {
  let answer: Receiver<Vec<RenderTextureReport>> = state.renderer.describe_textures(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no description", viewport.0),
  )
  .await
}
