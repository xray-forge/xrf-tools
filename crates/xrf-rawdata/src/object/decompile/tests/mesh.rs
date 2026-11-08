use xrf_editable_object::{EditableSurface, EditableVMap, EditableVMapKind, EditableVMapReference};
use xrf_error::XrfResult;
use xrf_skeleton::SkeletonJointLimit;

use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::tests::fixtures::{DOWN, UP, assemble, corner, surface, with_links, with_uv};
use crate::object::joint_limit_sense::JointLimitSense;

#[test]
fn maps_a_seam_corner_into_the_polygon_uv_map_and_shares_references() -> XrfResult {
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
  // The second triangle maps point `a` elsewhere on the texture: a UV seam through a welded point.
  let triangles: Vec<[OgfCorner; 3]> = vec![
    [corner(a, UP), corner(b, UP), corner(c, UP)],
    [with_uv(corner(a, UP), [0.5, 0.5]), corner(c, UP), corner(d, UP)],
  ];

  let assembled: AssembledMesh = assemble(&[surface(triangles)])?;
  let vmaps: &[EditableVMap] = assembled.mesh.vmaps.as_deref().unwrap_or_default();

  assert_eq!(
    vmaps.iter().map(|it| (it.name.as_str(), it.kind)).collect::<Vec<_>>(),
    [
      ("texture", EditableVMapKind::Uv),
      ("link", EditableVMapKind::Weight),
      ("texture", EditableVMapKind::Uv),
    ]
  );
  assert_eq!(vmaps[0].get_count(), 4, "one UV per point");
  assert_eq!(vmaps[1].values, vec![1.0; 4], "a point following one bone weighs fully");
  assert_eq!(vmaps[2].vertex_indices, [0], "the seam corner's point");
  assert_eq!(vmaps[2].polygon_indices, Some(vec![1]), "and its face");
  // Five distinct corners, `c` shared by both triangles with one UV and one weight.
  assert_eq!(assembled.mesh.vmap_references.len(), 5);
  assert_eq!(
    assembled.mesh.faces[0].corners[2].vmap_reference,
    assembled.mesh.faces[1].corners[1].vmap_reference
  );

  Ok(())
}

#[test]
fn enters_a_blended_point_in_the_weight_map_of_each_bone_it_follows() -> XrfResult {
  let [a, b, c] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
  // `a` follows the lid and the hinge, `b` the hinge alone, `c` the link; no point follows the lid alone.
  let triangles: Vec<[OgfCorner; 3]> = vec![[
    with_links(corner(a, UP), &[(2, 0.25), (1, 0.75)]),
    with_links(corner(b, UP), &[(2, 1.0)]),
    corner(c, UP),
  ]];

  let assembled: AssembledMesh = assemble(&[surface(triangles)])?;
  let vmaps: &[EditableVMap] = assembled.mesh.vmaps.as_deref().unwrap_or_default();
  let weights = |name: &str| {
    vmaps
      .iter()
      .find(|vmap| vmap.name == name)
      .map(|vmap| (vmap.vertex_indices.clone(), vmap.values.clone()))
  };

  // One weight map per bone some point follows, in bone order.
  assert_eq!(
    vmaps.iter().map(|it| it.name.as_str()).collect::<Vec<_>>(),
    ["texture", "link", "lid", "hinge"]
  );
  assert_eq!(weights("link"), Some((vec![2], vec![1.0])));
  assert_eq!(weights("lid"), Some((vec![0], vec![0.75])));
  assert_eq!(weights("hinge"), Some((vec![0, 1], vec![0.25, 1.0])));

  // The blended corner refers to its UV and to both of its weights, lid first by bone order.
  let reference: &EditableVMapReference =
    &assembled.mesh.vmap_references[assembled.mesh.faces[0].corners[0].vmap_reference as usize];

  assert_eq!(
    reference
      .points
      .iter()
      .map(|it| (it.vmap, it.index))
      .collect::<Vec<_>>(),
    [(0, 0), (2, 0), (3, 0)]
  );

  Ok(())
}

#[test]
fn splits_a_child_of_plain_and_two_sided_faces_into_two_surfaces() -> XrfResult {
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [2.0, 0.0, 0.0]];
  // One plain face, and one written with its mirror, all in one exported child.
  let triangles: Vec<[OgfCorner; 3]> = vec![
    [corner(a, UP), corner(b, UP), corner(c, UP)],
    [corner(b, UP), corner(d, UP), corner(c, UP)],
    [corner(c, DOWN), corner(d, DOWN), corner(b, DOWN)],
  ];

  let assembled: AssembledMesh = assemble(&[surface(triangles)])?;
  let surfaces: &[EditableSurface] = &assembled.surfaces;

  assert_eq!(surfaces.len(), 2);
  assert_eq!(surfaces[0].name, "wood_plank6");
  assert!(!surfaces[0].is_two_sided());
  assert_eq!(surfaces[1].name, "wood_plank6_2sided");
  assert!(surfaces[1].is_two_sided());
  assert_eq!(surfaces[1].compiler_shader, "default");
  assert_eq!(assembled.mesh.faces.len(), 2);
  assert_eq!(assembled.mesh.surface_faces[1].faces, [1]);

  Ok(())
}

#[test]
fn flips_a_joint_limit_between_the_stored_and_the_authored_sense() {
  // `SJointIKData::Export` writes `-max` then `-min`.
  let stored: SkeletonJointLimit = SkeletonJointLimit {
    min: -0.5,
    max: 0.2,
    spring_factor: 1.0,
    damping_factor: 2.0,
  };
  let authored: SkeletonJointLimit = JointLimitSense::flip(stored.clone());

  assert_eq!(
    authored,
    SkeletonJointLimit {
      min: -0.2,
      max: 0.5,
      ..stored
    }
  );
  assert_eq!(JointLimitSense::flip(authored), stored);
}
