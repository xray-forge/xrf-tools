use xrf_math::Vector3d;

use crate::weld::pack_grid::PackGrid;

/// A vertex is filed under its cell and under each cell its reach moves into, alone and together, never twice.
#[test]
fn files_a_vertex_under_its_cell_and_the_cells_its_reach_moves_into() {
  let mut grid: PackGrid = PackGrid::new([2, 2, 2]);

  grid.file(0, [1, 1, 1], [1, 1, 1]);
  grid.file(1, [0, 1, 1], [1, 1, 2]);

  assert_eq!(grid.get_filed([1, 1, 1]), [0, 1]);
  assert_eq!(grid.get_filed([0, 1, 1]), [1]);
  assert_eq!(grid.get_filed([0, 1, 2]), [1]);
  assert_eq!(grid.get_filed([1, 1, 2]), [1]);
  assert!(grid.get_filed([0, 0, 0]).is_empty());
}

/// The reach is half a cell, at most `EPS_L`.
#[test]
fn reaches_half_a_cell_at_most_a_millimetre() {
  let grid: PackGrid = PackGrid::new([4, 4, 4]);

  assert_eq!(
    grid.get_reach(&Vector3d::new(0.004, 100.0, 0.0)),
    Vector3d::new(0.0005, 0.001, 0.0)
  );
}
