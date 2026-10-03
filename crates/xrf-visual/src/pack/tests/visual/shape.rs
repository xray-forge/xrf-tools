//! Holds what a built body promises: faces seen from outside, a tangent basis running along its uvs, and a reason for a
//! mesh that packs nothing.

use xrf_math::Vector3d;

use crate::data::visual::geometry::visual_geometry::VisualGeometry;
use crate::pack::shape::visual_shape::VisualShape;
use crate::pack::tests::visual::reader::{read_f32_section, read_u16_section};
use crate::pack::visual::visual_mesh::VisualMesh;
use crate::pack::visual::visual_package::VisualPackage;
use crate::pack::visual::visual_packer::VisualPacker;

fn to_vectors(values: &[f32]) -> Vec<Vector3d> {
  values.as_chunks::<3>().0.iter().map(|it| Vector3d::from(*it)).collect()
}

/// Every triangle of a packed submesh, wound as stored, beside the normal of its first corner.
fn list_triangles(package: &VisualPackage, geometry: &VisualGeometry) -> Vec<([Vector3d; 3], Vector3d)> {
  let positions: Vec<Vector3d> = to_vectors(&read_f32_section(&package.buffer, geometry.positions));
  let normals: Vec<Vector3d> = to_vectors(&read_f32_section(&package.buffer, geometry.normals));
  let indices: Vec<u16> = read_u16_section(&package.buffer, geometry.indices);

  indices
    .as_chunks::<3>()
    .0
    .iter()
    .map(|triangle| {
      (
        triangle.map(|it| positions[usize::from(it)].clone()),
        normals[usize::from(triangle[0])].clone(),
      )
    })
    .collect()
}

#[test]
fn a_box_packs_a_face_a_submesh_each_wound_to_face_out() {
  let package: VisualPackage = VisualPacker::pack_meshes(&VisualShape::create_box(2.0, 1.0, 0.5));
  let outward: [Vector3d; 6] = [
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
  ]
  .map(Vector3d::from);

  assert_eq!(package.description.submeshes.len(), 6);

  for (submesh, outward) in package.description.submeshes.iter().zip(outward) {
    let geometry: &VisualGeometry = submesh.geometry().expect("expect every face to pack");

    for ([a, b, c], normal) in list_triangles(&package, geometry) {
      assert_eq!(normal, outward);
      // Counter-clockwise seen from outside, which is the front face a packed visual draws.
      assert!((&b - &a).cross(&(&c - &a)).dot(&outward) > 0.0);
    }
  }
}

#[test]
fn a_sphere_runs_its_tangent_along_u_at_right_angles_to_its_normal() {
  let package: VisualPackage = VisualPacker::pack_meshes(&[VisualShape::create_sphere(1.0, 16, 8)]);
  let geometry: &VisualGeometry = package.description.submeshes[0]
    .geometry()
    .expect("expect the sphere to pack");
  let normals: Vec<Vector3d> = to_vectors(&read_f32_section(&package.buffer, geometry.normals));
  let tangents: Vec<Vector3d> = to_vectors(&read_f32_section(&package.buffer, geometry.tangents));
  let binormals: Vec<Vector3d> = to_vectors(&read_f32_section(&package.buffer, geometry.binormals));

  for ((normal, tangent), binormal) in normals.iter().zip(&tangents).zip(&binormals) {
    assert!(normal.dot(tangent).abs() < 1e-4);
    assert!(normal.dot(binormal).abs() < 1e-4);
    assert!((tangent.length() - 1.0).abs() < 1e-4);
  }

  // On the equator at `u` zero the surface faces `-x`, and growing `u` turns it towards `+z`.
  let equator: usize = 4 * 17;

  assert!(tangents[equator].z > 0.9);
  assert!(binormals[equator].y < -0.9);
}

#[test]
fn a_mesh_without_whole_triangles_is_skipped_with_a_reason() {
  let mut mesh: VisualMesh = VisualShape::create_sphere(1.0, 4, 4);

  mesh.indices.pop();

  let package: VisualPackage = VisualPacker::pack_meshes(&[mesh]);

  assert_eq!(
    package.description.submeshes[0].skipped_reason(),
    Some("A built mesh has no whole triangles")
  );
}
