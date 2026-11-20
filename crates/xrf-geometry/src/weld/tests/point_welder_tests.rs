use xrf_math::{EPS_L, Vector3d};

use crate::weld::point_welder::PointWelder;

#[test]
fn welds_a_point_within_reach_on_every_axis_and_holds_one_beyond() {
  let mut welder: PointWelder = PointWelder::default();
  let first: u32 = welder.weld(&Vector3d::new(1.0, 2.0, 3.0));

  assert_eq!(
    welder.weld(&Vector3d::new(1.0 + EPS_L * 0.5, 2.0 - EPS_L * 0.5, 3.0)),
    first
  );
  assert_ne!(welder.weld(&Vector3d::new(1.0 + EPS_L * 2.0, 2.0, 3.0)), first);
  assert_eq!(welder.get_points().len(), 2);
  assert_eq!(welder.get_points()[first as usize], Vector3d::new(1.0, 2.0, 3.0));
}

#[test]
fn finds_a_held_point_across_a_cell_border() {
  let mut welder: PointWelder = PointWelder::default();
  // Either side of the border between two cells, within reach of each other.
  let below: u32 = welder.weld(&Vector3d::new(EPS_L * 0.9, 0.0, 0.0));

  assert_eq!(welder.weld(&Vector3d::new(EPS_L * 1.1, 0.0, 0.0)), below);
}
