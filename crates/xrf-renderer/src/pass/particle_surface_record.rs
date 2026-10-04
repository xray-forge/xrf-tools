/// How one particle effect's sprite is sampled and tested, as `shaders/frame/particles.wgsl` reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleSurfaceRecord {
  /// The sprite's base texture, by its bindless slot.
  pub texture: u32,
  pub flags: u32,
  /// The alpha a texel must exceed to be drawn, a fraction.
  pub alpha_reference: f32,
  /// The texture its `l_special` pass distorts with, by its bindless slot.
  pub distortion: u32,
}

impl ParticleSurfaceRecord {
  /// `Texture clamp`: sampled clamped to the edge rather than wrapped.
  pub const IS_CLAMPED: u32 = 1 << 0;
}
