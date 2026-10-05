use crate::alloc::UploadRing;
use crate::graph::pool::TransientPool;
use crate::graph::timing::GraphTimer;
use crate::param::{BindGroupCache, UniformBinding};
use crate::shader::ShaderType;

/// What frame graphs keep between frames: the pool their transients come from, the bind groups their passes bind, the
/// ring a frame's uniforms are pushed to, and the timer measuring their passes on the GPU.
pub struct GraphRuntime {
  pub pool: TransientPool,
  pub bind_groups: BindGroupCache,
  /// Pushed to while a frame is declared, flushed when it executes.
  pub uploads: UploadRing,
  pub timer: GraphTimer,
}

impl GraphRuntime {
  /// Bytes the upload ring starts with; it grows to what a frame pushes.
  const UPLOAD_CAPACITY: u64 = 16 * 1024;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    Self {
      pool: TransientPool::new(),
      bind_groups: BindGroupCache::new(),
      uploads: UploadRing::new(
        device,
        "graph uploads",
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
        Self::UPLOAD_CAPACITY,
      ),
      timer: GraphTimer::new(device, queue),
    }
  }

  /// Pushes `value` to the upload ring, for a pass to bind as a uniform this frame.
  pub fn push_uniform<T: ShaderType + bytemuck::Pod>(&mut self, value: &T) -> UniformBinding<T> {
    UniformBinding::new(self.uploads.push(value))
  }
}
