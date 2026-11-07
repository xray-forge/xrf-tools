use xrf_editable_object::EditableMesh;
use xrf_error::XrfResult;
use xrf_ogf::OgfVertexLink;

use crate::object::decompile::geometry::bone_influences::BoneInfluences;
use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::smoothing_inference::SmoothingInference;
use crate::object::decompile::geometry::two_sided_pairs::TwoSidedPairs;
use crate::object::decompile::geometry::welded_face::WeldedFace;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::tests::fixtures::{DOWN, OUT, UP, corner, link, surface, with_links};

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

  // The same position following another bone, or another set of bones, is another point.
  let mut other_bone: Vec<[OgfCorner; 3]> = quad(UP, UP);

  other_bone[1][0] = with_links(other_bone[1][0], &[(1, 1.0)]);

  assert_eq!(WeldedMesh::weld(&[surface(other_bone)])?.points.len(), 5);

  let mut blended: Vec<[OgfCorner; 3]> = quad(UP, UP);

  blended[1][0] = with_links(blended[1][0], &[(0, 0.5), (1, 0.5)]);

  assert_eq!(WeldedMesh::weld(&[surface(blended)])?.points.len(), 5);

  Ok(())
}

#[test]
fn undoes_the_padding_each_link_layout_writes() {
  let influences = |links: &[OgfVertexLink]| BoneInfluences::of_links(links).iter().collect::<Vec<(u16, f32)>>();

  // A one-bone vertex in a two-link child: the bone twice, the whole weight on the copy.
  assert_eq!(influences(&[link(4, 0.0), link(4, 1.0)]), vec![(4, 1.0)]);
  // Two bones in a three-link child: the first repeated last with what rounding leaves, a sliver either side of
  // nought, which joins the first bone's weight.
  for sliver in [-1e-8, 3e-8] {
    let collapsed: Vec<(u16, f32)> = influences(&[link(7, 0.75), link(2, 0.25), link(7, sliver)]);

    assert_eq!(collapsed.iter().map(|(bone, _)| *bone).collect::<Vec<u16>>(), [2, 7]);
    assert!((collapsed[1].1 - 0.75).abs() <= 1e-6, "{collapsed:?}");
  }
  // Three bones in a four-link child, the padding weightless.
  assert_eq!(
    influences(&[link(1, 0.5), link(3, 0.3), link(2, 0.2), link(1, 0.0)]),
    vec![(1, 0.5), (2, 0.2), (3, 0.3)]
  );
}

#[test]
fn welds_one_point_exported_into_children_of_different_layouts() -> XrfResult {
  // The same blended point, its weights rounded apart by two layouts, still welds into one.
  let [a, b, c] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
  let in_two_links: OgfCorner = with_links(corner(a, UP), &[(0, 0.700_000_05), (1, 0.299_999_95)]);
  let in_three_links: OgfCorner = with_links(corner(a, UP), &[(0, 0.7), (1, 0.3), (0, 1e-8)]);
  let welded: WeldedMesh = WeldedMesh::weld(&[
    surface(vec![[in_two_links, corner(b, UP), corner(c, UP)]]),
    surface(vec![[in_three_links, corner(c, UP), corner(b, UP)]]),
  ])?;

  assert_eq!(welded.points.len(), 3);

  Ok(())
}

#[test]
fn keeps_two_points_apart_where_the_export_moved_one_onto_the_other() -> XrfResult {
  // `VPack` snapped a source point onto its neighbour: one position, the same bones, weights of their own.
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [-1.0, 0.0, 0.0]];
  let welded: WeldedMesh = WeldedMesh::weld(&[surface(vec![
    [
      with_links(corner(a, UP), &[(0, 0.9), (1, 0.1)]),
      corner(b, UP),
      corner(c, UP),
    ],
    [
      with_links(corner(a, UP), &[(0, 0.75), (1, 0.25)]),
      corner(c, UP),
      corner(d, UP),
    ],
  ])])?;

  assert_eq!(welded.points.len(), 5);
  assert_eq!(
    welded.point_influences[3].iter().collect::<Vec<_>>(),
    [(0, 0.75), (1, 0.25)]
  );

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
    vec![EditableMesh::FLAT_FACE_SMOOTHING, EditableMesh::FLAT_FACE_SMOOTHING]
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
