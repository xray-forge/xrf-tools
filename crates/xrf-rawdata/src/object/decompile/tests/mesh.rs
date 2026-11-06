use xrf_editable_object::{EditableSurface, EditableVMap, EditableVMapKind};
use xrf_error::XrfResult;
use xrf_skeleton::SkeletonJointLimit;

use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::skeleton::bone_converter::BoneConverter;
use crate::object::decompile::tests::fixtures::{DOWN, UP, assemble, corner, surface, with_uv};

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
  assert_eq!(vmaps[1].values, vec![1.0; 4], "a rigid model weighs every point fully");
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
fn turns_a_stored_joint_limit_back_into_the_authored_one() {
  // `SJointIKData::Export` writes `-max` then `-min`.
  let stored: SkeletonJointLimit = SkeletonJointLimit {
    min: -0.5,
    max: 0.2,
    spring_factor: 1.0,
    damping_factor: 2.0,
  };

  assert_eq!(
    BoneConverter::to_authored_limit(stored.clone()),
    SkeletonJointLimit {
      min: -0.2,
      max: 0.5,
      ..stored
    }
  );
}
