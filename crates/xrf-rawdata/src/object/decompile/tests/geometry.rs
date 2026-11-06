use xrf_error::XrfResult;

use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::smoothing_inference::SmoothingInference;
use crate::object::decompile::geometry::two_sided_pairs::TwoSidedPairs;
use crate::object::decompile::geometry::welded_face::WeldedFace;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::tests::fixtures::{DOWN, OUT, UP, corner, surface};

/// Two triangles of a flat quad, `a b c` and `a c d`, each triangle's corners carrying its own normal.
fn quad(first_normal: [f32; 3], second_normal: [f32; 3]) -> Vec<[OgfCorner; 3]> {
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];

  vec![
    [
      corner(a, first_normal),
      corner(b, first_normal),
      corner(c, first_normal),
    ],
    [
      corner(a, second_normal),
      corner(c, second_normal),
      corner(d, second_normal),
    ],
  ]
}

#[test]
fn welds_corners_split_from_one_point_and_keeps_points_of_two_bones_apart() -> XrfResult {
  // The two triangles' shared corners carry different normals, as a hard edge's do, yet weld into shared points.
  let welded: WeldedMesh = WeldedMesh::weld(&[surface(quad(UP, OUT))])?;

  assert_eq!(welded.points.len(), 4);
  assert_eq!(welded.faces[0].points, [0, 1, 2]);
  assert_eq!(welded.faces[1].points, [0, 2, 3]);

  // The same position owned by another bone is another point.
  let mut other_bone: Vec<[OgfCorner; 3]> = quad(UP, UP);

  other_bone[1][0].bone = 1;

  assert_eq!(WeldedMesh::weld(&[surface(other_bone)])?.points.len(), 5);

  Ok(())
}

#[test]
fn folds_a_mirrored_copy_into_one_two_sided_face_and_keeps_a_plain_duplicate() -> XrfResult {
  let [a, b, c] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
  let face: [OgfCorner; 3] = [corner(a, UP), corner(b, UP), corner(c, UP)];
  let mirror: [OgfCorner; 3] = [corner(c, DOWN), corner(b, DOWN), corner(a, DOWN)];

  let folded: Vec<WeldedFace> = TwoSidedPairs::fold(WeldedMesh::weld(&[surface(vec![face, mirror])])?.faces);

  assert_eq!(folded.len(), 1);
  assert!(folded[0].is_two_sided);
  assert_eq!(folded[0].points, [0, 1, 2], "the face written first is the one kept");

  // The same face twice, wound alike, is no two-sided pair.
  let doubled: Vec<WeldedFace> = TwoSidedPairs::fold(WeldedMesh::weld(&[surface(vec![face, face])])?.faces);

  assert_eq!(doubled.len(), 2);
  assert!(doubled.iter().all(|it| !it.is_two_sided));

  Ok(())
}

#[test]
fn smooths_an_edge_whose_ends_carry_one_normal_and_leaves_the_rest_hard() -> XrfResult {
  // One normal at the shared edge's ends: its edge 2 (`c a`) in the first face, edge 0 (`a c`) in the second, soft.
  let smooth: WeldedMesh = WeldedMesh::weld(&[surface(quad(UP, UP))])?;

  assert_eq!(SmoothingInference::infer(&smooth.faces), vec![0b011, 0b110]);

  // Normals that differ across it make it hard, and both faces flat.
  let hard: WeldedMesh = WeldedMesh::weld(&[surface(quad(UP, OUT))])?;

  assert_eq!(
    SmoothingInference::infer(&hard.faces),
    vec![SmoothingInference::FLAT, SmoothingInference::FLAT]
  );

  Ok(())
}

#[test]
fn takes_normals_that_differ_in_their_last_bits_for_one_smoothed_normal() {
  let a: OgfCorner = corner([0.0; 3], [0.0, 1.0, 0.0]);
  let b: OgfCorner = corner([0.0; 3], [0.000_2, 0.999_9, 0.0]);

  assert!(a.has_normal_of(&b));
  assert!(!a.has_normal_of(&corner([0.0; 3], OUT)));
  assert!(a.has_normal_opposite(&corner([0.0; 3], DOWN)));
}
