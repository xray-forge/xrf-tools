use glam::Vec3;

/// How hard it rains, in the terms a weather keyframe uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderRainfall {
  /// `rain_color`.
  pub color: Vec3,
  /// `rain_density`, in `(0, 1]`.
  pub density: f32,
}
