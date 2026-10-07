use glam::UVec4;
use xrf_renderer_core::ShaderStruct;

/// The texture slots a terrain surface lays over its base: four details and their bumps, weighed by a mask.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TerrainSlots")]
pub struct StaticTerrainSlots {
  /// The details the mask's red, green, blue and alpha weigh.
  pub details: UVec4,
  /// Their bumps, in the same order.
  pub bumps: UVec4,
  pub mask: u32,
  pub _pad: [u32; 3],
}

impl StaticTerrainSlots {
  /// Every texture slot it samples: the details, their bumps and the mask.
  pub fn iter_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self
      .details
      .to_array()
      .into_iter()
      .chain(self.bumps.to_array())
      .chain([self.mask])
  }
}
