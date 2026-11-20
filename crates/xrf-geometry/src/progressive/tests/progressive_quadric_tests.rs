use crate::progressive::progressive_quadric::ProgressiveQuadric;

const FIRST: [f64; 5] = [0.0, 0.0, 0.0, 0.0, 0.0];
const SECOND: [f64; 5] = [1.0, 0.0, 0.0, 1.0, 0.0];
const THIRD: [f64; 5] = [0.0, 0.0, 1.0, 0.0, 1.0];

#[test]
fn test_a_point_on_the_triangles_plane_has_no_error() {
  let quadric: ProgressiveQuadric = ProgressiveQuadric::of_triangle(&FIRST, &SECOND, &THIRD);

  for point in [
    FIRST,
    SECOND,
    THIRD,
    [0.25, 0.0, 0.25, 0.25, 0.25],
    [3.0, 0.0, -2.0, 3.0, -2.0],
  ] {
    assert!(quadric.evaluate(&point).abs() < 1e-12, "{point:?} lies on the plane");
  }
}

#[test]
fn test_the_error_is_the_squared_distance_off_the_plane() {
  let quadric: ProgressiveQuadric = ProgressiveQuadric::of_triangle(&FIRST, &SECOND, &THIRD);

  assert!(
    (quadric.evaluate(&[0.25, 2.0, 0.25, 0.25, 0.25]) - 4.0).abs() < 1e-12,
    "two units above, in position"
  );
  assert!(
    (quadric.evaluate(&[0.25, 0.0, 0.25, 0.25, 0.75]) - 0.125).abs() < 1e-12,
    "half off, in coordinate"
  );
}

#[test]
fn test_quadrics_sum() {
  let mut sum: ProgressiveQuadric = ProgressiveQuadric::ZERO;
  let floor: ProgressiveQuadric = ProgressiveQuadric::of_triangle(&FIRST, &SECOND, &THIRD);

  sum.add(&floor);
  sum.add(&floor);

  assert!((sum.evaluate(&[0.0, 1.0, 0.0, 0.0, 0.0]) - 2.0).abs() < 1e-12);
}
