use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_renderer::{RenderFrameReport, RenderViewportId};

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::state::RenderState;

/// Say what a viewport's frames cost when it last reported them, as its `Frame` events do, for a caller polling rather
/// than listening; none before its first report.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_frame"))]
#[tauri::command(rename = "describe_frame")]
pub async fn render_describe_frame(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
) -> TauriResult<Option<RenderFrameReport>> {
  let answer: Receiver<Option<RenderFrameReport>> = state.renderer.describe_frame(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no frame", viewport.0),
  )
  .await
}
