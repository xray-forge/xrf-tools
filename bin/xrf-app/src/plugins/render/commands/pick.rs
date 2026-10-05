use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_error::XrfResult;
use xrf_renderer::{RenderLevelHit, RenderPick, RenderViewportId};

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::state::RenderState;

/// Name what a viewport's level draws under a point, css pixels from its corner, or nothing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pick"))]
#[tauri::command(rename = "pick")]
pub async fn render_pick(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
  x: f32,
  y: f32,
) -> TauriResult<Option<RenderLevelHit>> {
  let answer: Receiver<XrfResult<RenderPick>> = state.renderer.pick(viewport, x, y);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} drew no frame to pick in", viewport.0),
  )
  .await?
  .map(|pick| pick.hit)
  .map_err(|error| error.to_string())
}
