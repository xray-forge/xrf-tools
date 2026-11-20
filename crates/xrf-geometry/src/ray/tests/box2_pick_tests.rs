use crate::ray::box2_pick::Box2Pick;

#[test]
fn crosses_a_box_the_line_runs_through_either_way() {
  assert!(Box2Pick::is_crossing((2.0, 0.0), (3.0, 1.0), (0.0, 0.5), (1.0, 0.0)));
  assert!(
    Box2Pick::is_crossing((2.0, 0.0), (3.0, 1.0), (5.0, 0.5), (1.0, 0.0)),
    "behind the start"
  );
  assert!(
    Box2Pick::is_crossing((2.0, 2.0), (3.0, 3.0), (0.0, 0.0), (1.0, 1.0)),
    "along a diagonal"
  );
}

#[test]
fn misses_a_box_beside_the_line() {
  assert!(!Box2Pick::is_crossing((2.0, 0.0), (3.0, 1.0), (0.0, 5.0), (1.0, 0.0)));
  assert!(!Box2Pick::is_crossing((2.0, 0.0), (3.0, 1.0), (0.0, 5.0), (0.0, 1.0)));
  assert!(
    !Box2Pick::is_crossing((2.0, 0.0), (3.0, 1.0), (0.0, 0.5), (0.0, 0.0)),
    "no direction"
  );
}
