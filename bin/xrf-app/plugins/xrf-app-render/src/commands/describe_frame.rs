use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::types::TauriResult;
use xrf_renderer::{RenderFrameReport, RenderViewportId};

use crate::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::renderer::Renderer;

/// Say what a viewport's frames cost when it last reported them, as its `Frame` events do, for a caller polling rather
/// than listening; none before its first report.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_frame"))]
#[tauri::command(rename = "describe_frame")]
pub async fn render_describe_frame(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
) -> TauriResult<Option<RenderFrameReport>> {
  let answer: Receiver<Option<RenderFrameReport>> = renderer.describe_frame(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no frame", viewport.0),
  )
  .await
}
