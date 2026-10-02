use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::lighting::sun_view_ray::SunViewRay;

/// The view's four edges, which the sun's cascades are placed along one after another: each cascade starts where the
/// edges leave the one before it (`render_phase_sun.cpp`, the `rays` each cascade hands the next).
#[derive(Clone, Copy, Debug)]
pub struct SunViewRays {
  pub rays: [SunViewRay; 4],
  /// The same edges as they start, at the near plane, which every cascade holds the first stretch of.
  pub near: [SunViewRay; 4],
}

impl SunViewRays {
  /// The edges starting at a view's near plane, its corners numbered `-x -y`, `+x -y`, `-x +y`, `+x +y`.
  pub fn new(view: &CameraView) -> Self {
    let inverse: Mat4 = view.get_view_projection().inverse();
    let corner = |index: usize| -> SunViewRay {
      let x: f32 = if index & 1 != 0 { 1.0 } else { -1.0 };
      let y: f32 = if index & 2 != 0 { 1.0 } else { -1.0 };
      // Depth is reversed: one is the near plane.
      let point: Vec4 = inverse * Vec4::new(x, y, 1.0, 1.0);
      let origin: Vec3 = point.truncate() / point.w;

      SunViewRay {
        origin,
        direction: (origin - view.position).normalize_or_zero(),
      }
    };
    let rays: [SunViewRay; 4] = [corner(0), corner(1), corner(2), corner(3)];

    Self { rays, near: rays }
  }
}
