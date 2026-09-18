use xrf_db::{OgfBox, OgfSphere};
use xrf_math::Vector3d;

use crate::data::visual_bounds::{VisualBounds, VisualBox, VisualSphere};

/// Convert one X-Ray position or direction into three.js space.
pub fn convert_vector(vector: &Vector3d) -> Vector3d {
  Vector3d {
    x: vector.x,
    y: vector.y,
    z: -vector.z,
  }
}

/// Convert one texture coordinate pair, which is to leave it alone.
pub fn convert_texture_coordinates(u: f32, v: f32) -> (f32, f32) {
  (u, v)
}

/// Convert the extent an OGF header declares, so it compares against measured geometry.
pub fn convert_declared_bounds(bounding_box: &OgfBox, bounding_sphere: &OgfSphere) -> VisualBounds {
  let min: Vector3d = convert_vector(&bounding_box.min);
  let max: Vector3d = convert_vector(&bounding_box.max);

  VisualBounds {
    bounding_box: VisualBox {
      min: Vector3d {
        x: min.x,
        y: min.y,
        z: min.z.min(max.z),
      },
      max: Vector3d {
        x: max.x,
        y: max.y,
        z: min.z.max(max.z),
      },
    },
    bounding_sphere: VisualSphere {
      center: convert_vector(&bounding_sphere.position),
      radius: bounding_sphere.radius,
    },
  }
}
