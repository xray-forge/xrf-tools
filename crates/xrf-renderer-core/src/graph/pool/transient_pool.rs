use std::collections::HashMap;

use crate::graph::pool::pooled_buffer::PooledBuffer;
use crate::graph::pool::pooled_texture::PooledTexture;
use crate::graph::pool::transient_buffer_key::TransientBufferKey;
use crate::graph::pool::transient_texture_key::TransientTextureKey;

/// The textures and buffers frame graphs draw their transients from, kept across frames by key; one left unused for
/// longer than the pool's patience is freed.
pub struct TransientPool {
  textures: HashMap<TransientTextureKey, Vec<PooledTexture>>,
  buffers: HashMap<TransientBufferKey, Vec<PooledBuffer>>,
  frame: u64,
  max_idle_frames: u64,
}

impl Default for TransientPool {
  fn default() -> Self {
    Self::new()
  }
}

impl TransientPool {
  /// Frames an unused texture or buffer is kept for before it is freed.
  pub const MAX_IDLE_FRAMES: u64 = 30;

  pub fn new() -> Self {
    Self {
      textures: HashMap::new(),
      buffers: HashMap::new(),
      frame: 0,
      max_idle_frames: Self::MAX_IDLE_FRAMES,
    }
  }

  pub fn with_max_idle_frames(mut self, frames: u64) -> Self {
    self.max_idle_frames = frames;
    self
  }

  /// Textures held, of every key.
  pub fn get_texture_count(&self) -> usize {
    self.textures.values().map(Vec::len).sum()
  }

  /// Buffers held, of every key.
  pub fn get_buffer_count(&self) -> usize {
    self.buffers.values().map(Vec::len).sum()
  }

  /// Starts a frame: frees what the frames before it left unused for too long.
  pub(crate) fn begin_frame(&mut self) {
    self.frame += 1;

    let (frame, patience) = (self.frame, self.max_idle_frames);

    for entries in self.textures.values_mut() {
      entries.retain(|entry| frame - entry.last_frame <= patience);
    }

    for entries in self.buffers.values_mut() {
      entries.retain(|entry| frame - entry.last_frame <= patience);
    }

    self.textures.retain(|_, entries| !entries.is_empty());
    self.buffers.retain(|_, entries| !entries.is_empty());
  }

  /// Makes sure `count` textures of a key exist, each marked used this frame.
  pub(crate) fn reserve_textures(&mut self, device: &wgpu::Device, key: TransientTextureKey, count: usize) {
    let frame: u64 = self.frame;
    let entries: &mut Vec<PooledTexture> = self.textures.entry(key).or_default();

    while entries.len() < count {
      let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("graph transient"),
        size: key.size,
        mip_level_count: key.mip_level_count,
        sample_count: key.sample_count,
        dimension: key.dimension,
        format: key.format,
        usage: key.usage,
        view_formats: &[],
      });
      let view: wgpu::TextureView = texture.create_view(&wgpu::TextureViewDescriptor::default());

      entries.push(PooledTexture {
        texture,
        view,
        last_frame: frame,
      });
    }

    for entry in entries.iter_mut().take(count) {
      entry.last_frame = frame;
    }
  }

  /// Makes sure `count` buffers of a key exist, each marked used this frame.
  pub(crate) fn reserve_buffers(&mut self, device: &wgpu::Device, key: TransientBufferKey, count: usize) {
    let frame: u64 = self.frame;
    let entries: &mut Vec<PooledBuffer> = self.buffers.entry(key).or_default();

    while entries.len() < count {
      entries.push(PooledBuffer {
        buffer: device.create_buffer(&wgpu::BufferDescriptor {
          label: Some("graph transient"),
          size: key.size,
          usage: key.usage,
          mapped_at_creation: false,
        }),
        last_frame: frame,
      });
    }

    for entry in entries.iter_mut().take(count) {
      entry.last_frame = frame;
    }
  }

  pub(crate) fn get_texture(&self, key: &TransientTextureKey, ordinal: usize) -> &PooledTexture {
    &self.textures[key][ordinal]
  }

  pub(crate) fn get_buffer(&self, key: &TransientBufferKey, ordinal: usize) -> &PooledBuffer {
    &self.buffers[key][ordinal]
  }
}
