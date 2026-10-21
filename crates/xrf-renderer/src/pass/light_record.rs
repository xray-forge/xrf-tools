use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// One local light standing in view this frame, in view space, as `shaders/common/light_clusters.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
pub struct LightRecord {
  /// Its position, then `1 / L_R²`, the falloff reaching zero at 95% of the range.
  pub position: Vec4,
  /// Its colour, then its specular weight.
  pub color: Vec4,
  /// Where it points, then `cos` of half its cone, or `LightRecord::NO_CONE`.
  pub axis: Vec4,
  /// Its right, then its projection's scale, `cot` of half the widened cone.
  pub right: Vec4,
  /// Its up, then its projector's texture slot, or `LightRecord::NO_PROJECTOR`.
  pub up: Vec4,
  /// The sphere it is binned by.
  pub sphere: Vec4,
  /// A shadowed light's near and far planes and its face count, all zero for one unshadowed; then its bulb's radius,
  /// squared (`get_bulb_radius`).
  pub shadow: Vec4,
  /// Each face's square of the shadow atlas in texture coordinates, then how far the face has faded.
  pub faces: [Vec4; 6],
  /// Whether its light scatters in the air, one for a shadowed light the level gives shafts (`has_shafts`) and nought
  /// for any other; then nothing.
  pub scatter: Vec4,
}

impl LightRecord {
  /// What a spot without a projector writes as its slot.
  pub const NO_PROJECTOR: f32 = -1.0;

  /// What a light without a cone writes as its cone's cosine: every point passes its test.
  pub const NO_CONE: f32 = -2.0;
}
