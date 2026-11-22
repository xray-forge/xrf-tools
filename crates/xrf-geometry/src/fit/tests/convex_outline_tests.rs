use xrf_math::glam::DVec2;

use crate::fit::convex_outline::ConvexOutline;

#[test]
fn outlines_a_square_counter_clockwise_without_inner_or_collinear_points() {
  let mut points: Vec<DVec2> = Vec::new();

  for x in 0..=4 {
    for y in 0..=4 {
      points.push(DVec2::new(f64::from(x), f64::from(y)));
    }
  }

  points.extend(points.clone());

  assert_eq!(
    ConvexOutline::of(&points),
    [
      DVec2::new(0.0, 0.0),
      DVec2::new(4.0, 0.0),
      DVec2::new(4.0, 4.0),
      DVec2::new(0.0, 4.0)
    ]
  );
}

#[test]
fn outlines_points_on_a_line_by_their_ends() {
  let points: Vec<DVec2> = (0..5)
    .map(|index| DVec2::new(f64::from(index), 2.0 * f64::from(index)))
    .collect();

  assert_eq!(ConvexOutline::of(&points), [DVec2::new(0.0, 0.0), DVec2::new(4.0, 8.0)]);
}

#[test]
fn gives_fewer_than_three_points_back_sorted() {
  let (a, b): (DVec2, DVec2) = (DVec2::new(1.0, 0.0), DVec2::new(0.0, 1.0));

  assert_eq!(ConvexOutline::of(&[a, b, a]), [b, a]);
  assert_eq!(ConvexOutline::of(&[a]), [a]);
  assert!(ConvexOutline::of(&[]).is_empty());
}
