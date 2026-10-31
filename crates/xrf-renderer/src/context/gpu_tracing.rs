use crate::context::gpu_requirements::can_trace;

/// Whether a GPU traces rays against the scene's geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuTracing {
  Available,
  Unavailable,
}

impl GpuTracing {
  /// What `device` offers.
  pub fn of(device: &wgpu::Device) -> Self {
    if can_trace(device) {
      Self::Available
    } else {
      Self::Unavailable
    }
  }

  pub fn is_available(self) -> bool {
    self == Self::Available
  }
}
