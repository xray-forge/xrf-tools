use std::collections::BTreeSet;

use glam::{DQuat, DVec3};

use crate::object::compile::bounds::convex_hull::ConvexHull;
use crate::object::compile::bounds::flush_box::FlushBox;

/// A unit cube's corners and a lattice inside it and on its faces, none of which may become a hull vertex.
fn cube_with_lattice() -> Vec<DVec3> {
  let mut points: Vec<DVec3> = (0..8)
    .map(|corner| {
      DVec3::new(
        f64::from(corner & 1),
        f64::from((corner >> 1) & 1),
        f64::from((corner >> 2) & 1),
      )
    })
    .collect();

  for x in 0..=4 {
    for y in 0..=4 {
      for z in 0..=4 {
        points.push(DVec3::new(f64::from(x), f64::from(y), f64::from(z)) / 4.0);
      }
    }
  }

  points
}

/// Every edge is shared by exactly two faces running it opposite ways: a closed, consistently wound surface.
fn assert_closed(faces: &[[usize; 3]]) {
  let mut edges: Vec<(usize, usize)> = faces
    .iter()
    .flat_map(|[a, b, c]| [(*a, *b), (*b, *c), (*c, *a)])
    .collect();

  edges.sort_unstable();

  for pair in edges.windows(2) {
    assert_ne!(pair[0], pair[1], "edge {:?} runs the same way twice", pair[0]);
  }

  for (a, b) in &edges {
    assert!(edges.binary_search(&(*b, *a)).is_ok(), "edge {a}-{b} has no twin");
  }
}

/// Every face looks away from the hull's centre, and every point lies on or behind every face.
fn assert_encloses(points: &[DVec3], hull: &ConvexHull) {
  let vertices: Vec<usize> = hull.vertices();
  let centre: DVec3 = vertices.iter().map(|it| points[*it]).sum::<DVec3>() / vertices.len() as f64;

  for [a, b, c] in &hull.faces {
    let normal: DVec3 = (points[*b] - points[*a]).cross(points[*c] - points[*a]).normalize();

    assert!(normal.dot(centre - points[*a]) < 0.0, "face {a}, {b}, {c} looks inward");

    for point in points {
      assert!(
        normal.dot(*point - points[*a]) <= 1e-9,
        "{point} lies outside face {a}, {b}, {c}"
      );
    }
  }
}

/// How far `point` lies outside the box.
fn outside(fitted: &FlushBox, point: DVec3) -> f64 {
  let offset: DVec3 = point - fitted.center;

  (0..3)
    .map(|axis| (offset.dot(fitted.axes[axis]).abs() - fitted.half_extents[axis]).max(0.0))
    .fold(0.0, f64::max)
}

#[test]
fn hulls_a_cube_with_a_lattice_to_its_corners() {
  let points: Vec<DVec3> = cube_with_lattice();
  let hull: ConvexHull = ConvexHull::of(&points).expect("a cube spans volume");
  let corners: BTreeSet<[u64; 3]> = hull
    .vertices()
    .iter()
    .map(|it| points[*it].to_array().map(f64::to_bits))
    .collect();

  assert_eq!(corners.len(), 8);
  assert_closed(&hull.faces);
  assert_encloses(&points, &hull);
}

#[test]
fn hulls_points_on_a_sphere_with_every_point_a_vertex() {
  let count: u32 = 400;
  let points: Vec<DVec3> = (0..count)
    .map(|index| {
      let height: f64 = 1.0 - 2.0 * (f64::from(index) + 0.5) / f64::from(count);
      let radius: f64 = (1.0 - height * height).sqrt();
      // The golden angle spreads the points evenly.
      let angle: f64 = f64::from(index) * 2.399_963_229_728_653;

      DVec3::new(radius * angle.cos(), height, radius * angle.sin())
    })
    .collect();
  let hull: ConvexHull = ConvexHull::of(&points).expect("a sphere spans volume");

  assert_eq!(hull.vertices().len(), 400);
  assert_eq!(hull.faces.len(), 2 * 400 - 4);
  assert_closed(&hull.faces);
  assert_encloses(&points, &hull);
}

#[test]
fn refuses_points_spanning_no_volume() {
  let flat: Vec<DVec3> = (0..20)
    .map(|index| DVec3::new(f64::from(index % 5), f64::from(index / 5), 0.0))
    .collect();
  let line: Vec<DVec3> = (0..20).map(|index| DVec3::new(f64::from(index), 0.0, 0.0)).collect();

  assert!(ConvexHull::of(&flat).is_none());
  assert!(ConvexHull::of(&line).is_none());
  assert!(ConvexHull::of(&[DVec3::ONE; 10]).is_none());
  assert!(ConvexHull::of(&cube_with_lattice()[..3]).is_none());
}

#[test]
fn gives_one_hull_for_one_input_whatever_its_duplicates() {
  let mut points: Vec<DVec3> = cube_with_lattice();

  points.extend(cube_with_lattice());

  let hull: ConvexHull = ConvexHull::of(&points).expect("a cube spans volume");

  assert_eq!(ConvexHull::of(&points).map(|it| it.faces), Some(hull.faces.clone()));
  assert_closed(&hull.faces);
  assert_encloses(&points, &hull);
}

#[test]
fn fits_a_turned_cuboid_exactly() {
  let turn: DQuat = DQuat::from_axis_angle(DVec3::new(1.0, 2.0, 3.0).normalize(), 0.7);
  let offset: DVec3 = DVec3::new(0.3, -1.2, 2.5);
  let points: Vec<DVec3> = cube_with_lattice()
    .iter()
    .map(|point| turn * (*point * DVec3::new(2.0, 1.0, 0.5)) + offset)
    .collect();
  let fitted: FlushBox = FlushBox::around(&points).expect("the cuboid has points");

  assert!((fitted.volume() - 1.0).abs() < 1e-9, "volume {}", fitted.volume());
  assert!((fitted.center - (turn * DVec3::new(1.0, 0.5, 0.25) + offset)).length() < 1e-9);

  for point in &points {
    assert!(outside(&fitted, *point) < 1e-12, "{point} lies outside");
  }

  // Orthonormal and right-handed, as the engine's rotation rows are taken.
  for axis in 0..3 {
    assert!((fitted.axes[axis].length() - 1.0).abs() < 1e-12);
    assert!(fitted.axes[axis].dot(fitted.axes[(axis + 1) % 3]).abs() < 1e-12);
  }

  assert!((fitted.axes[0].cross(fitted.axes[1]) - fitted.axes[2]).length() < 1e-12);
}

#[test]
fn fits_flat_points_with_no_thickness() {
  let points: Vec<DVec3> = (0..20)
    .map(|index| DVec3::new(f64::from(index % 5), 0.0, f64::from(index / 5)))
    .collect();
  let fitted: FlushBox = FlushBox::around(&points).expect("the points are there");
  let mut half: [f64; 3] = fitted.half_extents.to_array();

  half.sort_by(f64::total_cmp);

  assert!(half[0].abs() < 1e-12);
  assert!(
    (half[1] - 1.5).abs() < 1e-12 && (half[2] - 2.0).abs() < 1e-12,
    "{half:?}"
  );

  for point in &points {
    assert!(outside(&fitted, *point) < 1e-12);
  }
}

#[test]
fn fits_flat_points_by_area_whatever_their_principal_axes() {
  // A square with two corners doubled, which turns its principal axes onto the diagonals: every box around it has no
  // volume, and the one along its sides has half the area of the one along the diagonals.
  let corners: [DVec3; 4] = [DVec3::ZERO, DVec3::X, DVec3::X + DVec3::Z, DVec3::Z];
  let points: Vec<DVec3> = corners.iter().chain(&[corners[0], corners[2]]).copied().collect();
  let fitted: FlushBox = FlushBox::around(&points).expect("the points are there");
  let mut half: [f64; 3] = fitted.half_extents.to_array();

  half.sort_by(f64::total_cmp);

  assert!(half[0].abs() < 1e-12);
  assert!(
    (half[1] - 0.5).abs() < 1e-12 && (half[2] - 0.5).abs() < 1e-12,
    "{half:?}"
  );
}

#[test]
fn boxes_one_point_as_itself_and_no_point_as_nothing() {
  let point: DVec3 = DVec3::new(1.0, 2.0, 3.0);
  let fitted: FlushBox = FlushBox::around(&[point]).expect("the point is there");

  assert_eq!((fitted.center, fitted.half_extents), (point, DVec3::ZERO));
  assert!(FlushBox::around(&[]).is_none());
}
