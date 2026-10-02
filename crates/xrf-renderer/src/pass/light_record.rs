use glam::Vec4;

/// What a light without a cone writes as its cone's cosine: every point passes its test.
pub const LIGHT_NO_CONE: f32 = -2.0;

/// What a spot without a projector writes as its slot.
pub const LIGHT_NO_PROJECTOR: f32 = -1.0;

/// One local light standing in view this frame, in view space, as `shaders/frame/lights.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightRecord {
  /// Its position, then `1 / L_R²`, the falloff reaching zero at 95% of the range.
  pub position: Vec4,
  /// Its colour, then its specular weight.
  pub color: Vec4,
  /// Where it points, then `cos` of half its cone, or `LIGHT_NO_CONE`.
  pub axis: Vec4,
  /// Its right, then its projection's scale, `cot` of half the widened cone.
  pub right: Vec4,
  /// Its up, then its projector's texture slot, or `LIGHT_NO_PROJECTOR`.
  pub up: Vec4,
  /// The sphere it is binned by.
  pub sphere: Vec4,
  /// A shadowed light's near and far planes, its face count, and nothing: all zero for one unshadowed.
  pub shadow: Vec4,
  /// Each face's square of the shadow atlas in texture coordinates, then how far the face has faded.
  pub faces: [Vec4; 6],
}
