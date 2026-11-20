use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::capture::record::record_layout::RecordLayout;
use crate::frame::readback::gpu_readback::GpuReadback;

/// A mapped recorded frame for the recorder to read on its own thread; BGRA pixels at the recording's size.
///
/// Its buffer returns to the recording's ring when read or dropped, so a recorder that keeps frames makes later frames
/// drop.
#[derive(Debug)]
pub struct RecordFrame {
  readback: Arc<GpuReadback>,
  layout: RecordLayout,
  /// Its slot in sixtieths of a second since the recording's first frame.
  pub index: u64,
  /// When it was presented; where a video places it.
  pub presented: Instant,
  /// Frames dropped so far because every buffer was held.
  pub dropped: u64,
  is_read: bool,
}

impl RecordFrame {
  /// Interval between recorded frames, a sixtieth of a second; a frame shows this long while the next is missing.
  pub const INTERVAL: Duration = Duration::from_nanos(16_666_667);

  pub(crate) fn new(
    readback: Arc<GpuReadback>,
    layout: RecordLayout,
    index: u64,
    presented: Instant,
    dropped: u64,
  ) -> Self {
    Self {
      readback,
      layout,
      index,
      presented,
      dropped,
      is_read: false,
    }
  }

  /// Width and height in pixels.
  pub fn get_size(&self) -> (u32, u32) {
    self.layout.size
  }

  /// Reads its BGRA pixels, rows top down, and returns its buffer to the ring.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer could not be mapped.
  pub fn read(mut self) -> XrfResult<Vec<u8>> {
    let width: usize = self.layout.size.0 as usize;
    let height: usize = self.layout.size.1 as usize;
    let row: usize = self.layout.row as usize;

    self.is_read = true;
    self
      .readback
      .take(|bytes| {
        let mut pixels: Vec<u8> = Vec::with_capacity(width * height * 4);

        for line in bytes.chunks_exact(row).take(height) {
          pixels.extend_from_slice(&line[..width * 4]);
        }

        pixels
      })
      .unwrap_or_else(|| Err(XrfError::new_unexpected_error("A recorded frame was not back yet")))
  }
}

impl Drop for RecordFrame {
  fn drop(&mut self) {
    if !self.is_read {
      let _ = self.readback.take(|_| ());
    }
  }
}
