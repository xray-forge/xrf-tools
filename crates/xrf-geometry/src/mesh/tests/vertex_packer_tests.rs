use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector2d, Vector3d};

use crate::mesh::mesh_vertex::MeshVertex;
use crate::mesh::mesh_vertex_bone::MeshVertexBone;
use crate::mesh::vertex_packer::VertexPacker;
use crate::mesh::vertex_packing::VertexPacking;

const UP: [f32; 3] = [0.0, 1.0, 0.0];
const DOWN: [f32; 3] = [0.0, -1.0, 0.0];

#[test]
fn snaps_a_near_corner_onto_a_vertex_and_drops_degenerate_and_repeated_faces() -> XrfResult {
  let vertex = |position: [f32; 3], normal: [f32; 3]| MeshVertex {
    position: Vector3d::from(position),
    normal: Vector3d::from(normal),
    tangent: Vector3d::X,
    binormal: Vector3d::Z,
    texture_coordinate: Vector2d::ZERO,
    bones: vec![MeshVertexBone { bone: 0, weight: 1.0 }],
  };
  let mut packer: VertexPacker = VertexPacker::new(
    &Aabb {
      min: Vector3d::new(-1.0, -1.0, -1.0),
      max: Vector3d::new(1.0, 1.0, 1.0),
    },
    &VertexPacking::SKELETON,
  );
  let face = |first: [f32; 3], normal: [f32; 3]| {
    [
      vertex(first, normal),
      vertex([0.5, 0.0, 0.0], UP),
      vertex([0.0, 0.0, 0.5], UP),
    ]
  };

  packer.add_face(&mut face([0.0, 0.0, 0.0], UP))?;

  // Half a millimetre off with another normal: a vertex of its own, standing where the first one does.
  let mut near = face([0.0005, 0.0, 0.0], DOWN);

  packer.add_face(&mut near)?;

  assert_eq!(packer.get_vertices().len(), 4);
  assert_eq!(packer.get_vertices()[3].position, Vector3d::ZERO);
  assert_eq!(
    near[0].position,
    Vector3d::ZERO,
    "the corner to keep the position it was moved to"
  );

  // The first face again in another winding, and a face two of whose corners coincide.
  let [first, second, third] = face([0.0, 0.0, 0.0], UP);

  packer.add_face(&mut [second, third, first])?;
  packer.add_face(&mut [
    vertex([0.2, 0.0, 0.0], UP),
    vertex([0.2, 0.0, 0.0], UP),
    vertex([0.0, 0.0, 0.2], UP),
  ])?;

  assert_eq!((packer.get_faces().len(), packer.get_dropped_faces()), (2, 2));

  Ok(())
}

#[test]
fn refuses_a_corner_outside_the_box_it_packs_over() {
  let mut packer: VertexPacker = VertexPacker::new(
    &Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::new(1.0, 1.0, 1.0),
    },
    &VertexPacking::RIGID,
  );
  let corner = |x: f32| MeshVertex {
    position: Vector3d::new(x, 0.5, 0.5),
    normal: Vector3d::Y,
    tangent: Vector3d::X,
    binormal: Vector3d::Z,
    texture_coordinate: Vector2d::ZERO,
    bones: Vec::new(),
  };

  assert!(packer.add_face(&mut [corner(0.1), corner(0.5), corner(5.0)]).is_err());
}
