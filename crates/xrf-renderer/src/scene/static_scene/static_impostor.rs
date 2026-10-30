use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// One impostor as the cull and its draw read it: a tree seen from eight sides, as `static/records.wgsl` declares it.
/// Its corners are held apart, `CORNERS` of them an impostor in impostor order.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Impostor")]
pub struct StaticImpostor {
  /// Its tree's sphere in renderer space.
  pub sphere: Vec4,
  /// Each facet's normal, facing back along the direction it is seen from.
  pub normals: [Vec4; 8],
  /// `FLOD::lod_factor`, what its sphere's screen area is scaled by.
  pub factor: f32,
  /// Its surface row.
  pub surface: u32,
  /// Its tree's type, by its index in the level's types; `TreeTypes::NO_TYPE` for none.
  pub kind: u32,
  pub _pad: u32,
}

impl StaticImpostor {
  /// Facets an impostor has, and corners each of them.
  pub const FACETS: usize = 8;
  pub const CORNERS: usize = Self::FACETS * 4;
}
