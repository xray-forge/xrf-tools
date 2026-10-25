use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_renderer::{RenderLoadReport, RenderViewportId};

use crate::{RenderAnswer, Renderer};

/// Say how far the level a viewport was last asked to show has loaded, for a caller polling rather than listening; none
/// before its view is made.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_load")]
pub async fn render_describe_load(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
) -> TauriResult<Option<RenderLoadReport>> {
  let answer: Receiver<Option<RenderLoadReport>> = renderer.describe_load(viewport);

  RenderAnswer::await_viewport(answer, viewport, "load").await
}
