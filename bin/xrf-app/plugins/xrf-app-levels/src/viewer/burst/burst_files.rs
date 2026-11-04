use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use image::RgbaImage;
use xrf_app_core::TauriResult;
use xrf_app_render::CaptureFile;

/// A burst's pictures encoded as PNG and written on the worker pool, a few at once, so the burst's frames keep coming
/// while they are: a fullscreen frame's PNG takes about 200 ms to encode, its marks' 90 ms, longer than the renderer
/// takes to draw the next. A write that failed is answered by a later call.
pub struct BurstFiles {
  sender: Sender<TauriResult>,
  written: Receiver<TauriResult>,
  pending: usize,
}

impl BurstFiles {
  /// Pictures encoding at once before the next waits: four frames and their marks, about 230 MB at 3730×1930.
  const MOST_PENDING: usize = 8;

  pub fn new() -> Self {
    let (sender, written) = channel();

    Self {
      sender,
      written,
      pending: 0,
    }
  }

  /// Writes `picture` to `path` on the pool, its errors naming it `name`, waiting while the most are under way.
  ///
  /// # Errors
  ///
  /// Returns an error when a picture written before could not be.
  pub fn write(&mut self, picture: RgbaImage, path: PathBuf, name: &'static str) -> TauriResult {
    while self.pending >= Self::MOST_PENDING {
      self.wait_one()?;
    }

    let sender: Sender<TauriResult> = self.sender.clone();

    self.pending += 1;
    rayon::spawn(move || {
      let _ = sender.send(CaptureFile::write_png(&picture, &path, name));
    });

    Ok(())
  }

  /// Waits for every picture under way.
  ///
  /// # Errors
  ///
  /// Returns the first error of a picture that could not be written.
  pub fn finish(mut self) -> TauriResult {
    let mut result: TauriResult = Ok(());

    while self.pending > 0 {
      result = result.and(self.wait_one());
    }

    result
  }

  fn wait_one(&mut self) -> TauriResult {
    let written: TauriResult = self
      .written
      .recv()
      .map_err(|_| String::from("A burst's picture was lost as it was written"))?;

    self.pending -= 1;

    written
  }
}
