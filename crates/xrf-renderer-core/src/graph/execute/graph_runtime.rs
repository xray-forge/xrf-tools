use crate::graph::pool::TransientPool;
use crate::graph::timing::GraphTimer;
use crate::param::BindGroupCache;

/// What frame graphs keep between frames: the pool their transients come from, the bind groups their passes bind, and
/// the timer measuring their passes on the GPU.
pub struct GraphRuntime {
  pub pool: TransientPool,
  pub bind_groups: BindGroupCache,
  pub timer: GraphTimer,
}

impl GraphRuntime {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    Self {
      pool: TransientPool::new(),
      bind_groups: BindGroupCache::new(),
      timer: GraphTimer::new(device, queue),
    }
  }
}
