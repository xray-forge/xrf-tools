use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What one way of the enhanced water's reflection blur reads as its `WaterBlur`.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "WaterBlur")]
pub(crate) struct WaterBlurUniform {
  /// The way it blurs, then how many times its target's size the source's is.
  pub direction: Vec4,
}

impl WaterBlurUniform {
  /// Half-size texels a blurred texel reads at most away, either way: `frame/water_blur.wgsl`'s six taps, at most two
  /// texels apart, and one more for the filter.
  pub(crate) const REACH: u32 = 13;

  /// Across, from the reflection into a target half its size.
  pub(crate) const ACROSS: Self = Self {
    direction: Vec4::new(1.0, 0.0, 2.0, 0.0),
  };
  /// Down, between targets of one size.
  pub(crate) const DOWN: Self = Self {
    direction: Vec4::new(0.0, 1.0, 1.0, 0.0),
  };
}
