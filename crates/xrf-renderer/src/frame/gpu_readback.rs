use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use xrf_error::{XrfError, XrfResult};

/// Free for a copy, a copy recorded but not submitted, a map asked for, a map done, a map refused.
const FREE: u8 = 0;
const RECORDED: u8 = 1;
const MAPPING: u8 = 2;
const MAPPED: u8 = 3;
const FAILED: u8 = 4;

/// A buffer the GPU copies into and the CPU reads without ever waiting on it: a copy goes out with a frame, the buffer
/// is mapped once that frame's work is done, which a later frame's poll reports, and it is read then and freed again.
pub struct GpuReadback {
  buffer: wgpu::Buffer,
  state: Arc<AtomicU8>,
}

impl GpuReadback {
  pub fn new(device: &wgpu::Device, label: &str, size: u64) -> Self {
    Self {
      buffer: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
      }),
      state: Arc::new(AtomicU8::new(FREE)),
    }
  }

  pub fn get_buffer(&self) -> &wgpu::Buffer {
    &self.buffer
  }

  /// Whether a copy may go into it: nothing is on its way.
  pub fn is_free(&self) -> bool {
    self.state.load(Ordering::Acquire) == FREE
  }

  /// Notes a copy into it recorded with this frame's work.
  pub fn mark_recorded(&self) {
    self.state.store(RECORDED, Ordering::Release);
  }

  /// Asks for the copy recorded with the frame just submitted to be mapped.
  pub fn request(&self) {
    if self.state.load(Ordering::Acquire) == RECORDED {
      let state: Arc<AtomicU8> = Arc::clone(&self.state);

      self.state.store(MAPPING, Ordering::Release);
      self.buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        state.store(if result.is_ok() { MAPPED } else { FAILED }, Ordering::Release);
      });
    }
  }

  /// Reads what was copied once it is mapped, and frees the buffer for the next copy; nothing while it is on its way.
  pub fn take<R>(&self, read: impl FnOnce(&[u8]) -> R) -> Option<XrfResult<R>> {
    match self.state.load(Ordering::Acquire) {
      MAPPED => {
        let result: XrfResult<R> = self
          .buffer
          .slice(..)
          .get_mapped_range()
          .map(|view| read(&view))
          .map_err(|error| XrfError::new_unexpected_error(format!("Readback: {error}")));

        self.buffer.unmap();
        self.state.store(FREE, Ordering::Release);

        Some(result)
      }
      FAILED => {
        self.state.store(FREE, Ordering::Release);

        Some(Err(XrfError::new_unexpected_error(
          "Readback: the buffer could not be mapped",
        )))
      }
      _ => None,
    }
  }
}
