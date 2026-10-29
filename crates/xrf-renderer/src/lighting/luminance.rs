use glam::Vec3;

/// A linear colour's luminance, by Rec. 709's weights, as `luminance` in `common/luminance.wgsl` weighs it.
pub fn luminance(colour: Vec3) -> f32 {
  xrf_math::luminance(colour.to_array())
}
