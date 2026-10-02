/// One shader table entry as the static draws read it: its values, what it binds, and the slot of each texture.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticSurface {
  /// Times every texture repeats across the base coordinate.
  pub tiling: f32,
  /// Times the detail texture repeats per base repeat.
  pub detail_scale: f32,
  /// The alpha a cut-out texel must exceed, in `[0, 1]`.
  pub alpha_reference: f32,
  /// The lighting model's slice of the material table: `(material + 0.5) / 4`.
  pub slice: f32,
  /// Its flat colour, drawn where textures are off.
  pub color: [f32; 3],
  pub flags: u32,
  /// Texture slots, in [`StaticSurface::BASE`] order.
  pub textures: [u32; 8],
}

impl StaticSurface {
  pub const HAS_BASE: u32 = 1;
  pub const HAS_DETAIL: u32 = 1 << 1;
  pub const HAS_BUMP: u32 = 1 << 2;
  pub const HAS_DETAIL_BUMP: u32 = 1 << 3;
  /// A lightmap: hemisphere in alpha and sun in green, at the lightmap coordinate.
  pub const HAS_HEMI: u32 = 1 << 4;
  pub const IS_CUT_OUT: u32 = 1 << 5;
  /// Water's: one blending over the depth behind it, as `water_soft` and every Anomaly program does.
  pub const IS_SOFT_WATER: u32 = 1 << 6;
  /// Water drawn by one of Anomaly's programs, and the switches it defines.
  pub const IS_ANOMALY_WATER: u32 = 1 << 7;
  pub const IS_REFLECTING: u32 = 1 << 8;
  pub const IS_SPECULAR: u32 = 1 << 9;
  pub const IS_TRANSPARENT: u32 = 1 << 10;
  pub const IS_FOAMED: u32 = 1 << 11;
  /// Water's normal map, foam and distortion, in the slots a bumped surface binds its detail and bump pair.
  pub const HAS_WATER_NORMAL: u32 = 1 << 12;
  pub const HAS_FOAM: u32 = 1 << 13;
  pub const HAS_DISTORTION: u32 = 1 << 14;

  pub const BASE: usize = 0;
  pub const DETAIL: usize = 1;
  pub const BUMP: usize = 2;
  pub const BUMP_COMPANION: usize = 3;
  pub const DETAIL_BUMP: usize = 4;
  pub const DETAIL_BUMP_COMPANION: usize = 5;
  pub const HEMI: usize = 6;
  pub const WATER_NORMAL: usize = 1;
  pub const FOAM: usize = 2;
  pub const DISTORTION: usize = 3;
}
