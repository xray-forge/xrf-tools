use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_renderer::{RenderSurfaceGeometry, RenderViewportId};

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::state::RenderState;

/// Count what each shader table entry of a viewport's level draws across the sectors resident.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "measure_surfaces"))]
#[tauri::command(rename = "measure_surfaces")]
pub async fn render_measure_surfaces(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<RenderSurfaceGeometry>> {
  let answer: Receiver<Vec<RenderSurfaceGeometry>> = state.renderer.measure_surfaces(viewport);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no measure", viewport.0),
  )
  .await
}
