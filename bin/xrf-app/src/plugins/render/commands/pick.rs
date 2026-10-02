use std::sync::mpsc::Receiver;
use std::time::Duration;

use tauri::State;
use xrf_error::XrfResult;
use xrf_renderer::{RenderLevelHit, RenderViewportId};

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// How long a pick waits for the frame answering it: a viewport minimised or hidden draws none.
const PICK_TIMEOUT: Duration = Duration::from_secs(2);

/// Name what a viewport's level draws under a point, css pixels from its corner, or nothing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pick"))]
#[tauri::command(rename = "pick")]
pub async fn render_pick(
  state: State<'_, RenderState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  x: f32,
  y: f32,
) -> TauriResult<Option<RenderLevelHit>> {
  let answer: Receiver<XrfResult<Option<RenderLevelHit>>> = state.renderer.pick(viewport, x, y);

  execution
    .run_blocking(
      "Picking in a viewport",
      move || -> TauriResult<Option<RenderLevelHit>> {
        answer
          .recv_timeout(PICK_TIMEOUT)
          .map_err(|_| format!("Viewport {} drew no frame to pick in", viewport.0))?
          .map_err(|error| error.to_string())
      },
    )
    .await?
}
