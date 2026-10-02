use glam::Vec3;

/// `SMAP_adapt_min`, `SMAP_adapt_optimal` and `SMAP_adapt_max` (`r2_types.h`).
pub const LIGHT_SHADOW_MIN_SIZE: u32 = 32;
const OPTIMAL_SIZE: f32 = 768.0;
const MAX_SIZE: f32 = 1536.0;

/// The largest square a face takes, which leaves the atlas room for the rest.
pub const LIGHT_SHADOW_MAX_TILE: u32 = 1024;

/// How far every face's projection is widened past its cone: `tan_shift` (`Light_Render_Direct_ComputeXFS.cpp`).
pub const LIGHT_SHADOW_WIDENING: f32 = 3.5 * std::f32::consts::PI / 180.0;

/// An omni part's cone: a quarter turn.
pub const LIGHT_SHADOW_POINT_CONE: f32 = std::f32::consts::FRAC_PI_2;

/// A point light's six faces, `light::Export`'s omni parts, along the world's axes, each as where it looks and its up.
pub const LIGHT_SHADOW_POINT_FACES: [(Vec3, Vec3); 6] = [
  (Vec3::X, Vec3::Y),
  (Vec3::NEG_X, Vec3::Y),
  (Vec3::Y, Vec3::NEG_Z),
  (Vec3::NEG_Y, Vec3::Z),
  (Vec3::Z, Vec3::Y),
  (Vec3::NEG_Z, Vec3::Y),
];

/// `compute_xf_spot`'s map size, in texels: larger for a light nearer, brighter, facing the camera, longer and wider.
///
/// `distance` is the eye's to the light's spatial sphere; `duel` how far a spot faces the camera, one for a point.
pub fn to_light_shadow_size(range: f32, distance: f32, intensity: f32, duel: f32, cone: f32) -> f32 {
  let area: f32 = (range * range / (1.0 + distance * distance)).clamp(0.0, 1.0);
  let factor: f32 = area.sqrt()
    * intensity.max(0.0).powf(1.0 / 16.0)
    * duel.max(0.0).powf(1.0 / 4.0)
    * (range / 8.0).max(0.0).powf(1.0 / 4.0)
    * (cone / std::f32::consts::FRAC_PI_2).max(0.0).sqrt();

  (factor * OPTIMAL_SIZE)
    .floor()
    .clamp(LIGHT_SHADOW_MIN_SIZE as f32, MAX_SIZE)
}

/// The square a face is asked at: the power of two nearest its size, between the least and the largest a face takes.
pub fn to_light_shadow_tile_size(size: f32) -> u32 {
  let side: u32 = 1 << size.max(1.0).log2().round() as u32;

  side.clamp(LIGHT_SHADOW_MIN_SIZE, LIGHT_SHADOW_MAX_TILE)
}

/// What a view coordinate over its depth is scaled by to reach a face's clip space: `cot` of half the widened cone.
pub fn to_light_shadow_scale(cone: f32) -> f32 {
  1.0 / ((cone + LIGHT_SHADOW_WIDENING) / 2.0).tan()
}
