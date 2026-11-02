use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use image::{ImageFormat, RgbaImage};
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_error::XrfResult;
use xrf_renderer::RenderCapture;
use xrf_utils::StagedFile;

use crate::RenderAnswer;

/// A frame the renderer read back, written to a PNG file.
pub struct CaptureFile;

impl CaptureFile {
  /// How long a capture waits for the next frame: a viewport minimised or hidden draws none.
  const TIMEOUT: Duration = Duration::from_secs(5);

  /// Waits for a frame the renderer was asked to read back (`answer`) and writes it to `path` as a PNG; `missing` says
  /// what never came.
  ///
  /// # Errors
  ///
  /// Returns an error when no frame comes back, or when it cannot be encoded or written.
  pub async fn save(
    execution: &ExecutionState,
    answer: Receiver<XrfResult<RenderCapture>>,
    path: PathBuf,
    missing: String,
  ) -> TauriResult {
    let capture: RenderCapture = RenderAnswer::await_answer(answer, Self::TIMEOUT, missing)
      .await?
      .map_err(|error| error.to_string())?;

    execution
      .run_blocking("Saving a viewport's capture", move || -> TauriResult {
        let image: RgbaImage = RgbaImage::from_raw(capture.width, capture.height, capture.pixels)
          .ok_or_else(|| "The capture's pixels do not fill its size".to_string())?;

        Self::write_png(&image, &path, "capture")
      })
      .await?
  }

  /// Writes `picture` to `path` as a PNG whatever the path is named, encoded whole first so a failed write leaves no
  /// partial file; `name` says what it is in an error.
  ///
  /// # Errors
  ///
  /// Returns an error when it cannot be encoded or written.
  pub fn write_png(picture: &RgbaImage, path: &Path, name: &str) -> TauriResult {
    let mut png: Vec<u8> = Vec::new();

    picture
      .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
      .map_err(|error| format!("Failed to encode {name} '{}': {error}", path.display()))?;

    StagedFile::write(path, &png).map_err(|error| format!("Failed to write {name} '{}': {error}", path.display()))
  }
}
