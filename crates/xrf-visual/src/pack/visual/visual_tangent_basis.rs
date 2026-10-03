use xrf_math::Vector3d;

use crate::pack::visual::visual_mesh::VisualMesh;

/// Below this squared length a summed tangent has no direction worth keeping.
const DEGENERATE: f32 = 1e-12;

/// Every vertex's tangent and binormal: the directions its uvs' `u` and `v` grow along, summed over the triangles it
/// is a corner of, the tangent held in the surface and the binormal at right angles to both, pointing where `v` grows.
pub(crate) fn to_tangent_basis(mesh: &VisualMesh) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
  let count: usize = mesh.positions.len();
  let mut tangents: Vec<Vector3d> = vec![Vector3d::ZERO; count];
  let mut binormals: Vec<Vector3d> = vec![Vector3d::ZERO; count];

  for triangle in mesh.indices.as_chunks::<3>().0 {
    let [a, b, c] = triangle.map(usize::from);
    let origin: Vector3d = Vector3d::from(mesh.positions[a]);
    let edge1: Vector3d = &Vector3d::from(mesh.positions[b]) - &origin;
    let edge2: Vector3d = &Vector3d::from(mesh.positions[c]) - &origin;
    let (du1, dv1) = (mesh.uvs[b][0] - mesh.uvs[a][0], mesh.uvs[b][1] - mesh.uvs[a][1]);
    let (du2, dv2) = (mesh.uvs[c][0] - mesh.uvs[a][0], mesh.uvs[c][1] - mesh.uvs[a][1]);
    let determinant: f32 = du1 * dv2 - du2 * dv1;

    // A triangle collapsed in uv space, as a sphere's pole rows are, carries no direction to add.
    if determinant == 0.0 {
      continue;
    }

    let tangent: Vector3d = &(&(&edge1 * dv2) - &(&edge2 * dv1)) / determinant;
    let binormal: Vector3d = &(&(&edge2 * du1) - &(&edge1 * du2)) / determinant;

    for corner in [a, b, c] {
      tangents[corner] += &tangent;
      binormals[corner] += &binormal;
    }
  }

  (0..count)
    .map(|vertex| {
      let normal: Vector3d = Vector3d::from(mesh.normals[vertex]);
      let sum: &Vector3d = &tangents[vertex];
      let mut tangent: Vector3d = sum - &(&normal * normal.dot(sum));

      if tangent.length_squared() < DEGENERATE {
        // Nothing accumulated here, or what did cancelled out, so any direction in the surface will do.
        tangent = (if normal.x.abs() < 0.9 { Vector3d::X } else { Vector3d::Y }).cross(&normal);
      }

      let tangent: Vector3d = tangent.normalize_or_zero();
      let binormal: Vector3d = normal.cross(&tangent);
      let binormal: Vector3d = if binormal.dot(&binormals[vertex]) < 0.0 {
        -&binormal
      } else {
        binormal
      };

      (tangent.to_array(), binormal.to_array())
    })
    .unzip()
}
