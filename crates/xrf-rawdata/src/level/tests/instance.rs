//! Library objects found placed in a level's geometry: the fit, the search and what a placement claims.

use std::path::PathBuf;

use glam::{DMat3, DVec2, DVec3};

use crate::level::instance::level_instance_fit::LevelInstanceFit;
use crate::level::instance::level_instance_matcher::LevelInstanceMatcher;
use crate::level::instance::level_instance_triangles::LevelInstanceTriangles;
use crate::level::instance::level_library_object::{LevelLibraryObject, LevelLibraryTriangle};

/// A wedge of five faces, no two alike, so a placement fits it one way only: a 2 by 1 by 0.5 metre block cut on a slant.
fn wedge(texture: u32, uv_offset: f64) -> Vec<([DVec3; 3], [DVec2; 3])> {
  let p = |x: f64, y: f64, z: f64| DVec3::new(x, y, z);
  let uv = |u: f64, v: f64| DVec2::new(u * 0.37 + uv_offset, v * 0.61);
  let _ = texture;

  vec![
    (
      [p(0.0, 0.0, 0.0), p(2.0, 0.0, 0.0), p(2.0, 0.0, 1.0)],
      [uv(0.0, 0.0), uv(1.0, 0.0), uv(1.0, 1.0)],
    ),
    (
      [p(0.0, 0.0, 0.0), p(2.0, 0.0, 1.0), p(0.0, 0.0, 1.0)],
      [uv(0.0, 0.0), uv(1.0, 1.0), uv(0.0, 1.0)],
    ),
    (
      [p(0.0, 0.0, 0.0), p(0.0, 0.5, 0.0), p(2.0, 0.0, 0.0)],
      [uv(0.0, 0.2), uv(0.3, 0.2), uv(0.0, 0.9)],
    ),
    (
      [p(0.0, 0.0, 1.0), p(2.0, 0.0, 1.0), p(0.0, 0.5, 1.0)],
      [uv(0.5, 0.5), uv(0.9, 0.5), uv(0.5, 0.7)],
    ),
    (
      [p(0.0, 0.5, 0.0), p(0.0, 0.5, 1.0), p(2.0, 0.0, 1.0)],
      [uv(0.1, 0.1), uv(0.1, 0.8), uv(0.6, 0.8)],
    ),
  ]
}

fn object(reference: &str, texture: u32, uv_offset: f64) -> LevelLibraryObject {
  let mut triangles: Vec<LevelLibraryTriangle> = wedge(texture, uv_offset)
    .into_iter()
    .map(|(points, uvs)| LevelLibraryTriangle {
      texture,
      points,
      uvs: Some(uvs),
      is_two_sided: false,
      area: LevelLibraryTriangle::get_area(&points),
    })
    .collect();

  triangles.sort_by(|left, right| right.area.total_cmp(&left.area));

  LevelLibraryObject {
    reference: String::from(reference),
    path: PathBuf::new(),
    meshes: vec![String::from("mesh")],
    area: triangles.iter().map(|triangle| triangle.area).sum(),
    triangles,
    collision: Vec::new(),
  }
}

fn fit(degrees: f64, scale: f64, translation: DVec3) -> LevelInstanceFit {
  LevelInstanceFit {
    rotation: DMat3::from_rotation_y(degrees.to_radians()) * DMat3::from_rotation_x(0.3),
    scale,
    translation,
  }
}

/// `object` placed by `fit`, as a level holds it: its triangles in the level's space, where `is_split` its largest each
/// cut in four at its edges' middles, as xrLC cuts long faces and leaves the rest whole.
fn place(object: &LevelLibraryObject, fit: &LevelInstanceFit, is_split: bool) -> Vec<LevelLibraryTriangle> {
  object
    .triangles
    .iter()
    .flat_map(|triangle| {
      let points: [DVec3; 3] = triangle.points.map(|point| fit.apply(point));
      let uvs: [DVec2; 3] = triangle.uvs.expect("the wedge maps every corner");

      if !is_split || triangle.area < 1.0 {
        return vec![(points, uvs)];
      }

      let middle = |a: usize, b: usize| ((points[a] + points[b]) * 0.5, (uvs[a] + uvs[b]) * 0.5);
      let ([a, b, c], [ua, ub, uc]) = (points, uvs);
      let ((ab, uab), (bc, ubc), (ca, uca)) = (middle(0, 1), middle(1, 2), middle(2, 0));

      vec![
        ([a, ab, ca], [ua, uab, uca]),
        ([ab, b, bc], [uab, ub, ubc]),
        ([ca, bc, c], [uca, ubc, uc]),
        ([ab, bc, ca], [uab, ubc, uca]),
      ]
    })
    .map(|(points, uvs)| LevelLibraryTriangle {
      texture: triangle_texture(object),
      points,
      uvs: Some(uvs),
      is_two_sided: false,
      area: LevelLibraryTriangle::get_area(&points),
    })
    .collect()
}

fn triangle_texture(object: &LevelLibraryObject) -> u32 {
  object.triangles[0].texture
}

/// A level of `triangles`, indexed as the decompile indexes its static geometry.
fn level(triangles: Vec<LevelLibraryTriangle>) -> LevelInstanceTriangles {
  LevelInstanceTriangles::of_object(&LevelLibraryObject {
    reference: String::from("level"),
    path: PathBuf::new(),
    meshes: Vec::new(),
    area: triangles.iter().map(|triangle| triangle.area).sum(),
    triangles,
    collision: Vec::new(),
  })
}

fn assert_fit(found: &LevelInstanceFit, expected: &LevelInstanceFit, object: &LevelLibraryObject) {
  for point in object.triangles.iter().flat_map(|triangle| triangle.points) {
    assert!(
      found.apply(point).distance(expected.apply(point)) < 1e-6,
      "{found:?} places {point} away from {expected:?}"
    );
  }
}

#[test]
fn fits_a_rotation_scale_and_translation_from_matched_corners() {
  let expected: LevelInstanceFit = fit(37.0, 0.75, DVec3::new(10.0, -3.0, 42.0));
  let object: LevelLibraryObject = object("wedge", 0, 0.0);
  let pairs: Vec<(DVec3, DVec3)> = object
    .triangles
    .iter()
    .flat_map(|triangle| triangle.points)
    .map(|point| (point, expected.apply(point)))
    .collect();
  let triangle: &[DVec3; 3] = &object.triangles[0].points;

  assert_fit(
    &LevelInstanceFit::of_pairs(&pairs, true).expect("the pairs span a volume"),
    &expected,
    &object,
  );
  assert_fit(
    &LevelInstanceFit::of_triangles(triangle, &triangle.map(|point| expected.apply(point)), true)
      .expect("the triangle is not degenerate"),
    &expected,
    &object,
  );
}

#[test]
fn finds_each_placement_rigid_scaled_and_cut_and_claims_its_triangles() {
  let library: Vec<LevelLibraryObject> = vec![object("wedge", 0, 0.0)];
  let placements: [(LevelInstanceFit, bool); 3] = [
    (fit(0.0, 1.0, DVec3::new(5.0, 0.0, 5.0)), false),
    (fit(90.0, 1.0, DVec3::new(-20.0, 1.0, 3.0)), true),
    (fit(200.0, 0.8, DVec3::new(0.0, 4.0, -30.0)), false),
  ];
  let mut triangles: Vec<LevelLibraryTriangle> = placements
    .iter()
    .flat_map(|(fit, is_split)| place(&library[0], fit, *is_split))
    .collect();
  let count: usize = triangles.len();

  // Structure of the same texture beside them, which no placement may claim.
  triangles.push(LevelLibraryTriangle {
    texture: 0,
    points: [
      DVec3::new(50.0, 0.0, 0.0),
      DVec3::new(53.0, 0.0, 0.0),
      DVec3::new(50.0, 0.0, 7.0),
    ],
    uvs: Some([DVec2::ZERO, DVec2::X, DVec2::Y]),
    is_two_sided: false,
    area: 10.5,
  });

  let mut level: LevelInstanceTriangles = level(triangles);
  let found = LevelInstanceMatcher::find(&library, &mut level, LevelInstanceMatcher::COVERAGE);

  assert_eq!(found.len(), 3, "{found:?}");

  for (expected, _) in &placements {
    let placed = found
      .iter()
      .find(|it| it.fit.translation.distance(expected.translation) < 1e-3)
      .unwrap_or_else(|| panic!("no placement at {}", expected.translation));

    assert_fit(&placed.fit, expected, &library[0]);
    assert!(placed.coverage > 0.999, "{placed:?}");
  }

  assert_eq!(found.iter().map(|it| it.claimed.len()).sum::<usize>(), count);
  assert!(!level.triangles[count].is_claimed, "the structure was claimed");
}

#[test]
fn tells_apart_objects_of_one_shape_mapped_differently() {
  let library: Vec<LevelLibraryObject> = vec![object("sign_a", 0, 0.0), object("sign_b", 0, 0.5)];
  let triangles: Vec<LevelLibraryTriangle> = place(&library[1], &fit(10.0, 1.0, DVec3::new(1.0, 2.0, 3.0)), false);
  let mut level: LevelInstanceTriangles = level(triangles);
  let found = LevelInstanceMatcher::find(&library, &mut level, LevelInstanceMatcher::COVERAGE);

  assert_eq!(
    found
      .iter()
      .map(|it| library[it.object].reference.as_str())
      .collect::<Vec<_>>(),
    ["sign_b"]
  );
}

#[test]
fn leaves_a_copy_of_each_triangle_to_an_object_placed_on_the_same_spot() {
  let library: Vec<LevelLibraryObject> = vec![object("wedge", 0, 0.0)];
  let at: LevelInstanceFit = fit(45.0, 1.0, DVec3::new(3.0, 0.0, 3.0));
  let triangles: Vec<LevelLibraryTriangle> = [place(&library[0], &at, false), place(&library[0], &at, false)].concat();
  let mut level: LevelInstanceTriangles = level(triangles);
  let found = LevelInstanceMatcher::find(&library, &mut level, LevelInstanceMatcher::COVERAGE);

  assert_eq!(found.len(), 2, "{found:?}");
  assert!(level.triangles.iter().all(|triangle| triangle.is_claimed));
}
