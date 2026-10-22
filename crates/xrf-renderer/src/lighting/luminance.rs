use glam::Vec3;

/// A linear colour's luminance, by Rec. 709's weights, as `luminance` in `common/luminance.wgsl` weighs it.
pub fn luminance(colour: Vec3) -> f32 {
  colour.dot(Vec3::new(0.2126, 0.7152, 0.0722))
}
