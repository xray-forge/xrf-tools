use crate::pass::compute_grid::ComputeGrid;

#[test]
fn keeps_a_dispatch_that_fits_on_one_row() {
  assert_eq!(ComputeGrid::with_limit(65_535).get_size(65_535), (65_535, 1));
  assert_eq!(ComputeGrid::with_limit(65_535).get_size(0), (0, 1));
}

#[test]
fn lays_a_dispatch_past_the_limit_over_rows_covering_every_workgroup() {
  let (x, y): (u32, u32) = ComputeGrid::with_limit(65_535).get_size(65_536);

  assert_eq!((x, y), (65_535, 2));
  assert!(u64::from(x) * u64::from(y) >= 65_536);
}

#[test]
fn reads_no_limit_below_one() {
  assert_eq!(ComputeGrid::with_limit(0).get_size(3), (1, 3));
}
