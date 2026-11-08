use xrf_chunk::ChunkLayout;
use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector3d};

use crate::{EditableFace, EditableFaceCorner, EditableMesh, EditableMeshNormals};

/// A face's back-face bit, which renumbers its edges.
const BACK: u32 = 1 << 3;

/// Two triangles folded along their shared edge `a c`, the first's edge 2 and the second's edge 0, so their face
/// normals differ: `a b c` facing up and `a c d` tilted towards x.
fn hinge(smoothing_groups: Option<Vec<u32>>, flags: u8) -> EditableMesh {
  let face = |points: [u32; 3]| EditableFace {
    corners: points.map(|point| EditableFaceCorner {
      point,
      vmap_reference: 0,
    }),
  };

  EditableMesh {
    name: String::from("hinge"),
    bounds: Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::ZERO,
    },
    flags,
    options: None,
    points: vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(0.0, 0.0, 1.0),
      Vector3d::new(1.0, 0.0, 0.0),
      Vector3d::new(0.0, 1.0, -1.0),
    ],
    has_point_faces: false,
    faces: vec![face([0, 1, 2]), face([0, 2, 3])],
    smoothing_groups,
    corner_normals: None,
    vmap_references: Vec::new(),
    surface_faces: Vec::new(),
    vmaps: None,
    layout: ChunkLayout::default(),
  }
}

/// Whether the corner normal at `a`, corner 0 of the first face, is the first face's own.
fn is_flat_at_a(normals: &EditableMeshNormals) -> bool {
  (normals.corners[0][0].dot(&normals.faces[0]) - 1.0).abs() <= 1e-6
}

#[test]
fn takes_face_normals_by_the_editors_winding() -> XrfResult {
  let normals: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![u32::MAX; 2]), 0))?;

  // `(p1 - p0) × (p2 - p1)` of `a b c`: (0, 0, 1) × (1, 0, -1).
  assert_eq!(normals.faces[0], Vector3d::new(0.0, 1.0, 0.0));

  Ok(())
}

#[test]
fn smooths_across_an_edge_both_faces_call_soft_and_not_across_a_hard_one() -> XrfResult {
  let smooth: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![0, 0]), 0))?;
  let expected: Vector3d = (&smooth.faces[0] + &smooth.faces[1]).normalize_or_zero();

  assert!((smooth.corners[0][0].dot(&expected) - 1.0).abs() <= 1e-6);
  assert_eq!(
    smooth.corners[0][0], smooth.corners[1][0],
    "both faces share the normal at `a`"
  );

  // Hard on one side only is hard.
  let hard: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![1 << 2, 0]), 0))?;

  assert!(is_flat_at_a(&hard));

  Ok(())
}

#[test]
fn numbers_a_back_faces_edges_the_other_way_round() -> XrfResult {
  // On a back face edge `e` is stored at bit `(4 - e) % 3`: the first face's shared edge 2 stays at bit 2, and the
  // second face's shared edge 0 moves to bit 1, so marking bit 0 on it leaves the shared edge soft.
  let renamed: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![BACK, BACK | 1]), 0))?;

  assert!(!is_flat_at_a(&renamed));

  let hard: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![BACK, BACK | (1 << 1)]), 0))?;

  assert!(is_flat_at_a(&hard));

  // A front face and a back face never smooth together.
  let sides: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![0, BACK]), 0))?;

  assert!(is_flat_at_a(&sides));

  Ok(())
}

#[test]
fn refuses_smoothing_words_that_are_not_one_per_face() {
  assert!(EditableMeshNormals::of(&hinge(Some(vec![0]), 0)).is_err());
}

#[test]
fn smooths_a_mesh_without_smoothing_words_as_the_editor_loads_it() -> XrfResult {
  // `LoadMesh` fills every edge hard, or every edge soft under the older group mask.
  assert!(is_flat_at_a(&EditableMeshNormals::of(&hinge(None, 0))?));
  assert!(!is_flat_at_a(&EditableMeshNormals::of(&hinge(
    None,
    EditableMesh::SMOOTHING_GROUP_MASK_FLAG
  ))?));

  Ok(())
}
