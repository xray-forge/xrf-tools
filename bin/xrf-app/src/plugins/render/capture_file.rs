use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use image::{ImageFormat, RgbaImage};
use xrf_error::XrfResult;
use xrf_renderer::RenderCapture;
use xrf_utils::write_file_staged;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::await_answer;

/// How long a capture waits for the next frame: a viewport minimised or hidden draws none.
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);

/// Waits for a frame the renderer was asked to read back (`answer`) and writes it to `path` as a PNG; `missing` says
/// what never came.
///
/// # Errors
///
/// Returns an error when no frame comes back, or when it cannot be encoded or written.
pub async fn save_capture_file(
  execution: &ExecutionState,
  answer: Receiver<XrfResult<RenderCapture>>,
  path: PathBuf,
  missing: String,
) -> TauriResult {
  let capture: RenderCapture = await_answer(answer, CAPTURE_TIMEOUT, missing)
    .await?
    .map_err(|error| error.to_string())?;

  execution
    .run_blocking("Saving a viewport's capture", move || -> TauriResult {
      let image: RgbaImage = RgbaImage::from_raw(capture.width, capture.height, capture.pixels)
        .ok_or_else(|| "The capture's pixels do not fill its size".to_string())?;

      let mut png: Vec<u8> = Vec::new();

      image
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|error| format!("Failed to encode capture '{}': {error}", path.display()))?;

      // A PNG whatever the path is named, encoded whole first so a failed write leaves no partial file.
      write_file_staged(&path, &png).map_err(|error| format!("Failed to write capture '{}': {error}", path.display()))
    })
    .await?
}
