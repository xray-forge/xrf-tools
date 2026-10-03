use glam::Vec4;

/// How the grass sways this frame, as `shaders/grass/grass.wgsl` reads it, in the engine's space: each wave's lean across
/// the ground, then each wave's direction with its phase in `w`, both over a turn.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GrassWindUniform {
  pub wind_1: Vec4,
  pub wind_2: Vec4,
  pub wave_1: Vec4,
  pub wave_2: Vec4,
}
