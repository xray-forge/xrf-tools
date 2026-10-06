use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What one way of the enhanced water's reflection blur reads as its `WaterBlur`.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "WaterBlur")]
pub struct WaterBlurUniform {
  /// The way it blurs, then how many times its target's size the source's is.
  pub direction: Vec4,
}

impl WaterBlurUniform {
  /// Across, from the reflection into a target half its size.
  pub const ACROSS: Self = Self {
    direction: Vec4::new(1.0, 0.0, 2.0, 0.0),
  };
  /// Down, between targets of one size.
  pub const DOWN: Self = Self {
    direction: Vec4::new(0.0, 1.0, 1.0, 0.0),
  };
}
