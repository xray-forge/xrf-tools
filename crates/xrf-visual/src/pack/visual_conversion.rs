use xrf_math::{Aabb, Matrix4x4, Sphere, Vector3d};

use crate::data::visual::bounds::visual_bounds::VisualBounds;

/// Convert one X-Ray position or direction into renderer space, whose `z` runs the other way.
pub fn convert_vector(vector: &Vector3d) -> Vector3d {
  Vector3d {
    x: vector.x,
    y: vector.y,
    z: -vector.z,
  }
}

/// Convert one texture coordinate pair, which is to leave it alone.
pub fn convert_uvs(u: f32, v: f32) -> (f32, f32) {
  (u, v)
}

/// Convert the extent an OGF header declares, so it compares against measured geometry.
pub fn convert_declared_bounds(bounding_box: &Aabb, bounding_sphere: &Sphere) -> VisualBounds {
  let min: Vector3d = convert_vector(&bounding_box.min);
  let max: Vector3d = convert_vector(&bounding_box.max);

  VisualBounds {
    // Mirroring `z` swaps which corner is lowest along it, and only along it.
    bounding_box: Aabb {
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
    bounding_sphere: Sphere {
      center: convert_vector(&bounding_sphere.center),
      radius: bounding_sphere.radius,
    },
  }
}

/// Reverse the winding of every triangle in place.
pub(crate) fn reverse_triangle_winding<T>(indices: &mut [T]) {
  for triangle in indices.as_chunks_mut::<3>().0 {
    triangle.swap(1, 2);
  }
}

/// Convert a placement into renderer space, so it places already converted vertices.
pub fn convert_placement(placement: &Matrix4x4) -> Matrix4x4 {
  const Z: usize = 2;

  let mut values: [f32; Matrix4x4::ELEMENTS] = placement.values;

  for row in 0..4 {
    for column in 0..4 {
      if (row == Z) != (column == Z) {
        values[row * 4 + column] = -values[row * 4 + column];
      }
    }
  }

  Matrix4x4 { values }
}
