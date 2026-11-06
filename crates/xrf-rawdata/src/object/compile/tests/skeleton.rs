use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector2d, Vector3d};

use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::compile::skeleton::breakable_parts::BreakableParts;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;
use crate::object::compile::skeleton::skeleton_vertex::{SkeletonVertex, SkeletonVertexBone};
use crate::object::compile::skeleton::vertex_packer::VertexPacker;
use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::tests::fixtures::{
  BONES, DOWN, UP, assemble, bone, chain, corner, object, surface, with_links,
};

/// Compiles the object the decompiler builds from one surface of `triangles`, on `bones`.
fn compile(
  triangles: Vec<[OgfCorner; 3]>,
  bones: Vec<xrf_editable_object::EditableBone>,
) -> XrfResult<SkeletonGeometry> {
  let object: EditableObjectFile = object(assemble(&[surface(triangles)])?, bones);

  ObjectCompiler::build_skeleton_geometry(&object)
}

/// A flat quad `a b c d`, its two faces sharing the edge `a c`.
const QUAD: [[f32; 3]; 4] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];

#[test]
fn packs_a_smooth_quad_into_four_vertices_padded_to_two_links() -> XrfResult {
  let [a, b, c, d] = QUAD;
  let geometry: SkeletonGeometry = compile(
    vec![
      [corner(a, UP), corner(b, UP), corner(c, UP)],
      [corner(a, UP), corner(c, UP), corner(d, UP)],
    ],
    chain(),
  )?;
  let split: &SkeletonSplit = &geometry.splits[0];

  assert_eq!(geometry.splits.len(), 1);
  assert_eq!((split.vertices.len(), split.faces.len(), split.link_count), (4, 2, 1));
  assert_eq!(split.faces, [[0, 1, 2], [0, 2, 3]]);
  assert_eq!(split.used_bones, [0]);
  assert!(split.vertices.iter().all(|vertex| vertex.bones
    == [
      SkeletonVertexBone { bone: 0, weight: 1.0 },
      SkeletonVertexBone { bone: 0, weight: 0.0 }
    ]));

  Ok(())
}

#[test]
fn keeps_a_corner_of_a_one_link_face_apart_from_the_same_corner_of_a_blended_one() -> XrfResult {
  let [a, b, c, d] = QUAD;
  // `d` blends two bones, so the second face lists `a` and `c` once, the first face twice.
  let geometry: SkeletonGeometry = compile(
    vec![
      [corner(a, UP), corner(b, UP), corner(c, UP)],
      [
        corner(a, UP),
        corner(c, UP),
        with_links(corner(d, UP), &[(0, 0.5), (1, 0.5)]),
      ],
    ],
    chain(),
  )?;
  let split: &SkeletonSplit = &geometry.splits[0];

  assert_eq!((split.vertices.len(), split.link_count), (6, 2));
  assert_eq!(split.used_bones, [0, 1]);
  assert_eq!(split.vertices[3].bones, [SkeletonVertexBone { bone: 0, weight: 1.0 }]);
  assert_eq!(
    split.vertices[5].bones,
    [
      SkeletonVertexBone { bone: 0, weight: 0.5 },
      SkeletonVertexBone { bone: 1, weight: 0.5 }
    ]
  );

  Ok(())
}

#[test]
fn adds_a_two_sided_face_again_mirrored() -> XrfResult {
  let [a, b, c, _] = QUAD;
  let geometry: SkeletonGeometry = compile(
    vec![
      [corner(a, UP), corner(b, UP), corner(c, UP)],
      [corner(c, DOWN), corner(b, DOWN), corner(a, DOWN)],
    ],
    chain(),
  )?;
  let split: &SkeletonSplit = &geometry.splits[0];
  let position = |face: usize, corner: usize| split.vertices[usize::from(split.faces[face][corner])].position;
  let normal = |face: usize, corner: usize| split.vertices[usize::from(split.faces[face][corner])].normal;

  assert_eq!((split.faces.len(), split.vertices.len()), (2, 6));
  assert_eq!(
    [position(1, 0), position(1, 1), position(1, 2)],
    [position(0, 0), position(0, 2), position(0, 1)]
  );
  assert_eq!(normal(1, 0), -&normal(0, 0));

  Ok(())
}

#[test]
fn refuses_a_face_spanning_two_breakable_parts() -> XrfResult {
  let [a, b, c, _] = QUAD;
  let bones = vec![
    bone(BONES[0], "", false),
    bone(BONES[1], BONES[0], true),
    bone(BONES[2], BONES[1], false),
  ];
  let triangles: Vec<[OgfCorner; 3]> = vec![[
    corner(a, UP),
    corner(b, UP),
    with_links(corner(c, UP), &[(0, 0.5), (1, 0.5)]),
  ]];

  assert!(compile(triangles, bones).is_err());

  Ok(())
}

#[test]
fn numbers_breakable_parts_down_from_the_root() {
  let bones = vec![
    bone("root", "", false),
    bone("arm", "root", true),
    bone("hand", "arm", false),
    bone("door", "root", true),
  ];

  assert_eq!(BreakableParts::of(&bones), [0, 1, 1, 2]);
  assert_eq!(BreakableParts::of(&chain()), [0, 0, 0]);
}

#[test]
fn snaps_a_near_corner_onto_a_vertex_and_drops_degenerate_and_repeated_faces() -> XrfResult {
  let vertex = |position: [f32; 3], normal: [f32; 3]| SkeletonVertex {
    position: Vector3d::from(position),
    normal: Vector3d::from(normal),
    tangent: Vector3d::X,
    binormal: Vector3d::Z,
    texture_coordinate: Vector2d::ZERO,
    bones: vec![SkeletonVertexBone { bone: 0, weight: 1.0 }],
  };
  let mut packer: VertexPacker = VertexPacker::new(&Aabb {
    min: Vector3d::new(-1.0, -1.0, -1.0),
    max: Vector3d::new(1.0, 1.0, 1.0),
  });
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

  assert_eq!(packer.vertices.len(), 4);
  assert_eq!(packer.vertices[3].position, Vector3d::ZERO);
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

  assert_eq!((packer.faces.len(), packer.dropped_faces), (2, 2));

  Ok(())
}
