use crate::alloc::UploadRing;
use crate::graph::pool::{AttachmentViewCache, TransientPool};
use crate::graph::timing::GraphTimer;
use crate::param::{BindGroupCache, UniformBinding};
use crate::shader::ShaderType;

/// What frame graphs keep between frames: the pool their transients come from, the bind groups their passes bind, the
/// views of the mips and layers they draw into, the ring a frame's uniforms are pushed to, and the timer measuring
/// their passes on the GPU.
pub struct GraphRuntime {
  pub pool: TransientPool,
  pub bind_groups: BindGroupCache,
  pub attachment_views: AttachmentViewCache,
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
      attachment_views: AttachmentViewCache::default(),
      uploads: UploadRing::new(
        device,
        "graph uploads",
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
        Self::UPLOAD_CAPACITY,
      ),
      timer: GraphTimer::new(device, queue),
    }
  }

  /// Drops the pooled transients, bind groups and attachment views, for a renderer drawing no frames.
  ///
  /// Their patience counts frames, so without frames they would hold what the last frames drew into for good; the next
  /// frame makes again what it needs.
  pub fn forget_frames(&mut self) {
    self.pool.clear();
    self.bind_groups.clear_groups();
    self.attachment_views.clear();
  }

  /// Drops at once the pooled transients, bind groups and attachment views the frame just executed did not use.
  ///
  /// For a frame drawn at new sizes: what the old sizes keyed is not used again, and a set of it a frame, as while a
  /// window is dragged, can exhaust GPU memory and slow frames to a few a second before patience ages it out.
  pub fn forget_unused(&mut self) {
    self.pool.forget_unused();
    self.bind_groups.forget_unused();
    self.attachment_views.forget_unused();
  }

  /// Pushes `value` to the upload ring, for a pass to bind as a uniform this frame.
  pub fn push_uniform<T: ShaderType + bytemuck::Pod>(&mut self, value: &T) -> UniformBinding<T> {
    UniformBinding::new(self.uploads.push(value))
  }
}
