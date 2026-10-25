use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_renderer::{RenderFrameReport, RenderViewportId};

use crate::{RenderAnswer, Renderer};

/// Say what a viewport's frames cost when it last reported them, as its `Frame` events do, for a caller polling rather
/// than listening; none before its first report.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_frame")]
pub async fn render_describe_frame(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
) -> TauriResult<Option<RenderFrameReport>> {
  let answer: Receiver<Option<RenderFrameReport>> = renderer.describe_frame(viewport);

  RenderAnswer::await_viewport(answer, viewport, "frame").await
}
