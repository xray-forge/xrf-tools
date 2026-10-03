use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_renderer::{RenderLevelProblems, RenderViewportId};

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::state::RenderState;

/// Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_problems"))]
#[tauri::command(rename = "describe_problems")]
pub async fn render_describe_problems(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
) -> TauriResult<RenderLevelProblems> {
  let answer: Receiver<RenderLevelProblems> = state.renderer.describe_problems(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no description", viewport.0),
  )
  .await
}
