use xrf_error::XrfResult;
use xrf_math::{Vector2d, Vector3d};

use crate::mesh::mesh::Mesh;
use crate::mesh::mesh_vertex::MeshVertex;
use crate::mesh::mesh_vertex_bone::MeshVertexBone;
use crate::mesh::tangent_basis::TangentBasis;

/// A vertex on the floor, facing up, at `position` and mapped at `uv`.
fn vertex(position: [f32; 3], uv: [f32; 2]) -> MeshVertex {
  MeshVertex {
    position: Vector3d::from(position),
    normal: Vector3d::Y,
    tangent: Vector3d::X,
    binormal: Vector3d::Z,
    texture_coordinate: Vector2d::from(uv),
    bones: vec![MeshVertexBone { bone: 0, weight: 1.0 }],
  }
}

fn is_near(first: &Vector3d, second: &Vector3d) -> bool {
  first.distance(second) <= 1e-5
}

#[test]
fn runs_the_tangent_along_u_and_the_binormal_along_v() -> XrfResult {
  // A square on the floor, `u` along `x` and `v` along `z`.
  let Mesh { vertices, faces } = TangentBasis::mend(Mesh {
    vertices: vec![
      vertex([0.0, 0.0, 0.0], [0.0, 0.0]),
      vertex([1.0, 0.0, 0.0], [1.0, 0.0]),
      vertex([1.0, 0.0, 1.0], [1.0, 1.0]),
      vertex([0.0, 0.0, 1.0], [0.0, 1.0]),
    ],
    faces: vec![[0, 1, 2], [0, 2, 3]],
  })?;

  assert_eq!(vertices.len(), 4);
  assert_eq!(faces, vec![[0, 1, 2], [0, 2, 3]]);
  assert!(vertices.iter().all(|vertex| is_near(&vertex.tangent, &Vector3d::X)));
  assert!(vertices.iter().all(|vertex| is_near(&vertex.binormal, &Vector3d::Z)));

  Ok(())
}

#[test]
fn splits_the_vertices_of_an_edge_the_texture_is_mirrored_across() -> XrfResult {
  // The second face runs `u` against `x`: its tangent opposes the first's, so the two corners of the shared edge
  // `a c` are split, one vertex for each face.
  let Mesh { vertices, faces } = TangentBasis::mend(Mesh {
    vertices: vec![
      vertex([0.0, 0.0, 0.0], [0.0, 0.0]),
      vertex([1.0, 0.0, 0.0], [1.0, 0.0]),
      vertex([1.0, 0.0, 1.0], [1.0, 1.0]),
      vertex([0.0, 0.0, 1.0], [2.0, 1.0]),
    ],
    faces: vec![[0, 1, 2], [0, 2, 3]],
  })?;

  assert_eq!(vertices.len(), 6);
  assert_ne!(faces[0][0], faces[1][0], "`a` split");
  assert_ne!(faces[0][2], faces[1][1], "`c` split");
  assert!(is_near(&vertices[faces[0][0] as usize].tangent, &Vector3d::X));
  assert!(is_near(&vertices[faces[1][0] as usize].tangent, &-&Vector3d::X));

  Ok(())
}

#[test]
fn moves_texture_coordinates_by_whole_tiles_towards_the_origin() -> XrfResult {
  let mesh: Mesh<u32> = TangentBasis::mend(Mesh {
    vertices: vec![
      vertex([0.0, 0.0, 0.0], [5.25, -3.5]),
      vertex([1.0, 0.0, 0.0], [6.25, -3.5]),
      vertex([1.0, 0.0, 1.0], [6.25, -2.5]),
    ],
    faces: vec![[0, 1, 2]],
  })?;

  // `u` spans 5.25 to 6.25, its middle 5.75, so 5 tiles come off. `v` spans -3.5 to -2.5, but the span's top starts
  // from the smallest positive float, as the export's does, so its middle is about -1.75 and 2 tiles go on.
  assert_eq!(mesh.vertices[0].texture_coordinate, Vector2d::new(0.25, -1.5));

  Ok(())
}
