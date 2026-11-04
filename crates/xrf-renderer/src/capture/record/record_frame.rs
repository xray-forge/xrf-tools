use std::sync::Arc;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};

use crate::frame::readback::gpu_readback::GpuReadback;

/// A recorded frame back from the GPU and mapped, handed to the recorder to read on its own thread: its buffer goes back
/// to the recording's ring once read, or once this is dropped unread, so a recorder that keeps frames holds buffers and
/// the frames after them drop. Its pixels are four bytes, blue, green, red, alpha, at the recording's size.
#[derive(Debug)]
pub struct RecordFrame {
  readback: Arc<GpuReadback>,
  width: u32,
  height: u32,
  /// Bytes a row of the copy takes, aligned as copies need.
  row: u32,
  /// Its place among the recording's sixtieths of a second, counted from its first frame.
  pub index: u64,
  /// When it was presented, which a video places it at.
  pub presented: Instant,
  /// Frames the recording dropped so far, its buffers all held.
  pub dropped: u64,
  is_read: bool,
}

impl RecordFrame {
  pub(crate) fn new(
    readback: Arc<GpuReadback>,
    (width, height, row): (u32, u32, u32),
    index: u64,
    presented: Instant,
    dropped: u64,
  ) -> Self {
    Self {
      readback,
      width,
      height,
      row,
      index,
      presented,
      dropped,
      is_read: false,
    }
  }

  /// Pixels across and down.
  pub fn get_size(&self) -> (u32, u32) {
    (self.width, self.height)
  }

  /// Its pixels, rows top down, each four bytes blue first; its buffer goes back to the ring.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer could not be mapped.
  pub fn read(mut self) -> XrfResult<Vec<u8>> {
    let (width, height, row): (usize, usize, usize) = (self.width as usize, self.height as usize, self.row as usize);

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
