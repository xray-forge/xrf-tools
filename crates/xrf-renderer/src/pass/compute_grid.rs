/// Lays a dispatch's workgroups over two dimensions once one dimension would pass the device's limit, which is 65,535
/// on D3D12 and the least Vulkan guarantees. Its shaders read their index with `compute_index` (`common/compute_grid`).
#[derive(Clone, Copy, Debug)]
pub struct ComputeGrid {
  limit: u32,
}

impl ComputeGrid {
  pub fn new(device: &wgpu::Device) -> Self {
    Self::with_limit(device.limits().max_compute_workgroups_per_dimension)
  }

  pub fn with_limit(limit: u32) -> Self {
    Self { limit: limit.max(1) }
  }

  /// The grid `groups` workgroups are dispatched as: one row while they fit the limit, else rows of the limit.
  pub fn get_size(self, groups: u32) -> (u32, u32) {
    if groups <= self.limit {
      (groups, 1)
    } else {
      (self.limit, groups.div_ceil(self.limit))
    }
  }

  /// Dispatches `groups` workgroups, laid over as many rows as the limit asks.
  pub fn dispatch(self, pass: &mut wgpu::ComputePass<'_>, groups: u32) {
    let (x, y): (u32, u32) = self.get_size(groups);

    pass.dispatch_workgroups(x, y, 1);
  }
}
