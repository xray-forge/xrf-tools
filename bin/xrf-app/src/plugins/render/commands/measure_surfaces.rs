use std::sync::mpsc::Receiver;
use std::time::Duration;

use tauri::State;
use xrf_renderer::{RenderSurfaceGeometry, RenderViewportId};

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// How long a measure waits for the render thread to take it up.
const MEASURE_TIMEOUT: Duration = Duration::from_secs(2);

/// Count what each shader table entry of a viewport's level draws across the sectors resident.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "measure_surfaces"))]
#[tauri::command(rename = "measure_surfaces")]
pub async fn render_measure_surfaces(
  state: State<'_, RenderState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<Vec<RenderSurfaceGeometry>> {
  let answer: Receiver<Vec<RenderSurfaceGeometry>> = state.renderer.measure_surfaces(viewport);

  execution
    .run_blocking(
      "Measuring a viewport's surfaces",
      move || -> TauriResult<Vec<RenderSurfaceGeometry>> {
        answer
          .recv_timeout(MEASURE_TIMEOUT)
          .map_err(|_| format!("Viewport {} answered no measure", viewport.0))
      },
    )
    .await?
}
