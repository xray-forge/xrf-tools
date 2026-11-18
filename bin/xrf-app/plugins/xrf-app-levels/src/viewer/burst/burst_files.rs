use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use image::RgbaImage;
use xrf_app_core::TauriResult;
use xrf_app_render::CaptureFile;

/// Writes a burst's pictures as PNG on the worker pool, a few at once, so frames keep coming meanwhile: a fullscreen
/// frame's PNG takes about 200 ms to encode and its marks' 90 ms, longer than the next frame takes to draw. A failed
/// write is reported by a later call.
pub(crate) struct BurstFiles {
  sender: Sender<TauriResult>,
  written: Receiver<TauriResult>,
  pending: usize,
}

impl BurstFiles {
  /// Pictures encoding at once: four frames and their marks, about 230 MB at 3730×1930.
  const MOST_PENDING: usize = 8;

  pub(crate) fn new() -> Self {
    let (sender, written) = channel();

    Self {
      sender,
      written,
      pending: 0,
    }
  }

  /// Write `picture` to `path` on the pool, named `name` in errors; waits while [`Self::MOST_PENDING`] are under way.
  ///
  /// # Errors
  ///
  /// Returns an error when an earlier picture could not be written.
  pub(crate) fn write(&mut self, picture: RgbaImage, path: PathBuf, name: &'static str) -> TauriResult {
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

  /// Wait for every picture under way.
  ///
  /// # Errors
  ///
  /// Returns the first picture's write error.
  pub(crate) fn finish(mut self) -> TauriResult {
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
