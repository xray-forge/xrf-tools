use xrf_math::{Matrix4x4, Vector3d};
use xrf_ogf::{OgfBox, OgfSphere, OgfTreeDefinitionChunk};

use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::data::visual::bounds::visual_box::VisualBox;
use crate::data::visual::bounds::visual_sphere::VisualSphere;

/// Convert one X-Ray position or direction into three.js space.
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

/// Reverse the winding of every triangle in place.
pub(crate) fn reverse_triangle_winding<T>(indices: &mut [T]) {
  for triangle in indices.as_chunks_mut::<3>().0 {
    triangle.swap(1, 2);
  }
}

/// Convert a placement into renderer space, so it places already converted vertices.
/// `ps_r__Tree_SBC`'s default, the correction a tree's colour terms are bound with (`xrRender_console.cpp`).
const TREE_SCALE_BIAS_CORRECTION: f32 = 1.5;

/// What `FTreeVisual::Render` multiplies that correction by under every deferred renderer.
const DEFERRED_TREE_CORRECTION: f32 = 1.3333;

/// A tree's hemisphere terms as `FTreeVisual` binds them, `c_scale.w` then `c_bias.w`: what scales and then offsets
/// its vertices' hemisphere byte.
pub fn convert_tree_hemi(tree: &OgfTreeDefinitionChunk) -> [f32; 2] {
  let correction: f32 = TREE_SCALE_BIAS_CORRECTION * DEFERRED_TREE_CORRECTION * OgfTreeDefinitionChunk::COLOR_SCALE;

  [tree.scale.hemi * correction, tree.bias.hemi * correction]
}

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
