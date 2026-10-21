use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What filling one mip of the prefiltered sky cube reads as its `SkyPrefilter`: the mip, the roughness it stands for,
/// the directions sampled a texel, and its texels a side; then the first mip's texels a side, and one where it fills the
/// sky alone's cube (`SkyPrefilter::alone`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "SkyPrefilter")]
pub struct SkyPrefilterUniform {
  pub level: Vec4,
  pub shape: Vec4,
}

impl SkyPrefilterUniform {
  /// Mip `level` of `levels`, `size` texels a side over a first mip `first` a side, sampling `samples` directions, of
  /// the whole cube or of the sky alone's (`is_alone`).
  pub fn new(level: u32, levels: u32, (size, first): (u32, u32), samples: u32, is_alone: bool) -> Self {
    Self {
      level: Vec4::new(
        level as f32,
        level as f32 / (levels - 1).max(1) as f32,
        samples as f32,
        size as f32,
      ),
      shape: Vec4::new(first as f32, is_alone as u32 as f32, 0.0, 0.0),
    }
  }
}
