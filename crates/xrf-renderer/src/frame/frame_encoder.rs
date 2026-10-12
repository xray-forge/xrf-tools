use std::ops::{Deref, DerefMut};

use wgpu::util::StagingBelt;

/// The frame's own encoder, submitted before the frame graph's groups, and the staging belt its writes into existing
/// buffers go through: each write is a copy out of a staging chunk the belt reuses frame after frame, where a queue's
/// `write_buffer` makes a staging buffer of its own every call. A write lands where it is recorded among the encoder's
/// commands, so it is recorded before whatever of this encoder reads it.
pub struct FrameEncoder {
  encoder: wgpu::CommandEncoder,
  belt: StagingBelt,
}

impl FrameEncoder {
  /// Bytes of each staging chunk: more than any one write but the largest a frame makes, which takes a chunk of its
  /// own.
  pub const CHUNK: u64 = 1 << 20;

  pub fn new(device: &wgpu::Device, belt: StagingBelt) -> Self {
    Self {
      encoder: device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") }),
      belt,
    }
  }

  /// A staging belt for frames of `device`.
  pub fn create_belt(device: &wgpu::Device) -> StagingBelt {
    StagingBelt::new(device.clone(), Self::CHUNK)
  }

  /// Writes `bytes` into `target` at `offset`, both multiples of four bytes; nothing for none.
  pub fn write_buffer(&mut self, target: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
    if let Some(size) = wgpu::BufferSize::new(bytes.len() as u64) {
      self
        .belt
        .write_buffer(&mut self.encoder, target, offset, size)
        .copy_from_slice(bytes);
    }
  }

  /// Ends the frame's writes and its encoder: answers its commands, which must be submitted, and the belt, which takes
  /// its chunks back once they are.
  pub fn finish(mut self) -> (wgpu::CommandBuffer, StagingBelt) {
    self.belt.finish_and_recall_on_submit(&self.encoder);

    (self.encoder.finish(), self.belt)
  }
}

impl Deref for FrameEncoder {
  type Target = wgpu::CommandEncoder;

  fn deref(&self) -> &wgpu::CommandEncoder {
    &self.encoder
  }
}

impl DerefMut for FrameEncoder {
  fn deref_mut(&mut self) -> &mut wgpu::CommandEncoder {
    &mut self.encoder
  }
}
