use xrf_error::XrfResult;
use xrf_geometry::{Mesh, MeshVertex, MeshVertexBone, TangentBasis};
use xrf_math::{Vector2d, Vector3d};

use crate::object::compile::skeleton::mikktspace_basis::MikkTSpaceBasis;

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
fn gives_mikktspace_the_same_basis_where_the_mapping_is_square() -> XrfResult {
  // On a square mapped square every basis agrees: the binormal, `sign · (normal × tangent)`, runs along `v` as the
  // mender's does. MikkTSpace reads the sign off the winding, so the faces turn about their normal as the export's do,
  // `(p1 - p0) × (p2 - p1)` along it; the mender reads none.
  let Mesh { vertices, faces } = MikkTSpaceBasis::generate(Mesh {
    vertices: vec![
      vertex([0.0, 0.0, 0.0], [0.0, 0.0]),
      vertex([1.0, 0.0, 0.0], [1.0, 0.0]),
      vertex([1.0, 0.0, 1.0], [1.0, 1.0]),
      vertex([0.0, 0.0, 1.0], [0.0, 1.0]),
    ],
    faces: vec![[0, 2, 1], [0, 3, 2]],
  })?;

  assert_eq!(vertices.len(), 4);
  assert_eq!(faces, vec![[0, 1, 2], [0, 3, 1]]);
  assert!(vertices.iter().all(|vertex| is_near(&vertex.tangent, &Vector3d::X)));
  assert!(vertices.iter().all(|vertex| is_near(&vertex.binormal, &Vector3d::Z)));

  Ok(())
}

#[test]
fn splits_mikktspace_vertices_where_the_texture_is_mirrored() -> XrfResult {
  let Mesh { vertices, faces } = MikkTSpaceBasis::generate(Mesh {
    vertices: vec![
      vertex([0.0, 0.0, 0.0], [0.0, 0.0]),
      vertex([1.0, 0.0, 0.0], [1.0, 0.0]),
      vertex([1.0, 0.0, 1.0], [1.0, 1.0]),
      vertex([0.0, 0.0, 1.0], [2.0, 1.0]),
    ],
    faces: vec![[0, 2, 1], [0, 3, 2]],
  })?;

  assert_eq!(vertices.len(), 6);
  assert!(is_near(&vertices[faces[0][0] as usize].tangent, &Vector3d::X));
  assert!(is_near(&vertices[faces[1][0] as usize].tangent, &-&Vector3d::X));
  // The mirrored face's binormal still runs along `v`.
  assert!(is_near(&vertices[faces[1][0] as usize].binormal, &Vector3d::Z));

  Ok(())
}

#[test]
fn gives_a_face_mapped_to_a_point_a_basis_from_its_normal() -> XrfResult {
  // Every corner at one texture coordinate: MikkTSpace has no direction to give, and nothing near to borrow one from.
  let mesh: Mesh<u32> = MikkTSpaceBasis::generate(Mesh {
    vertices: vec![
      vertex([0.0, 0.0, 0.0], [0.5, 0.5]),
      vertex([1.0, 0.0, 0.0], [0.5, 0.5]),
      vertex([1.0, 0.0, 1.0], [0.5, 0.5]),
    ],
    faces: vec![[0, 1, 2]],
  })?;
  let (tangent, binormal) = TangentBasis::get_basis_from_normal(&Vector3d::Y);

  assert!(
    mesh
      .vertices
      .iter()
      .all(|vertex| { is_near(&vertex.tangent, &tangent) && is_near(&vertex.binormal, &binormal) })
  );

  Ok(())
}
