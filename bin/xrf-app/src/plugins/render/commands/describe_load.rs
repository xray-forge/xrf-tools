use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_renderer::{RenderLoadReport, RenderViewportId};

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::state::RenderState;

/// Say how far the level a viewport was last asked to show has loaded, for a caller polling rather than listening; none
/// before its view is made.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_load"))]
#[tauri::command(rename = "describe_load")]
pub async fn render_describe_load(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
) -> TauriResult<Option<RenderLoadReport>> {
  let answer: Receiver<Option<RenderLoadReport>> = state.renderer.describe_load(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no load", viewport.0),
  )
  .await
}
