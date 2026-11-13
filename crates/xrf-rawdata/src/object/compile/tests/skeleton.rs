use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_geometry::MeshVertexBone;

use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::compile::skeleton::breakable_parts::BreakableParts;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;
use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;
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
      MeshVertexBone { bone: 0, weight: 1.0 },
      MeshVertexBone { bone: 0, weight: 0.0 }
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
  assert_eq!(split.vertices[3].bones, [MeshVertexBone { bone: 0, weight: 1.0 }]);
  assert_eq!(
    split.vertices[5].bones,
    [
      MeshVertexBone { bone: 0, weight: 0.5 },
      MeshVertexBone { bone: 1, weight: 0.5 }
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
