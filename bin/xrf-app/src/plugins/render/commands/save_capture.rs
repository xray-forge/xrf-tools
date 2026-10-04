use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use image::RgbaImage;
use tauri::State;
use xrf_error::XrfResult;
use xrf_renderer::{RenderCapture, RenderViewportId};

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::await_answer;
use crate::plugins::render::state::RenderState;

/// How long a capture waits for the next frame: a viewport minimised or hidden draws none.
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);

/// Write a viewport's next presented frame to a PNG file, as the renderer drew it rather than as the screen shows it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_capture"))]
#[tauri::command(rename = "save_capture")]
pub async fn render_save_capture(
  state: State<'_, RenderState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  path: PathBuf,
) -> TauriResult {
  let answer: Receiver<XrfResult<RenderCapture>> = state.renderer.capture_viewport(viewport);

  let capture: RenderCapture = await_answer(
    answer,
    CAPTURE_TIMEOUT,
    format!("Viewport {} drew no frame to capture", viewport.0),
  )
  .await?
  .map_err(|error| error.to_string())?;

  execution
    .run_blocking("Saving a viewport's capture", move || -> TauriResult {
      let image: RgbaImage = RgbaImage::from_raw(capture.width, capture.height, capture.pixels)
        .ok_or_else(|| "The capture's pixels do not fill its size".to_string())?;

      image
        .save(&path)
        .map_err(|error| format!("Failed to write capture '{}': {error}", path.display()))
    })
    .await?
}
