/// The texture slots a terrain surface lays over its base: four details and their bumps, weighed by a mask.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticTerrainSlots {
  /// The details the mask's red, green, blue and alpha weigh.
  pub details: [u32; 4],
  /// Their bumps, in the same order.
  pub bumps: [u32; 4],
  pub mask: u32,
  pub pad: [u32; 3],
}

impl StaticTerrainSlots {
  /// Every texture slot it samples: the details, their bumps and the mask.
  pub fn iter_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self.details.iter().chain(&self.bumps).chain([&self.mask]).copied()
  }
}
