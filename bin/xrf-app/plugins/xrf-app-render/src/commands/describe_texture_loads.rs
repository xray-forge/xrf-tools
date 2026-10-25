use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_renderer::{RenderTextureReport, RenderViewportId};

use crate::{RenderAnswer, Renderer};

/// Say what became of every texture a viewport's level samples.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_texture_loads")]
pub async fn render_describe_texture_loads(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<RenderTextureReport>> {
  let answer: Receiver<Vec<RenderTextureReport>> = renderer.describe_texture_loads(viewport);

  RenderAnswer::await_viewport(answer, viewport, "description").await
}
