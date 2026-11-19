use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector3d};

use crate::ai::level_ai_grid::LevelAiGrid;

// 10 rows of 5 one-metre cells.
fn grid() -> XrfResult<LevelAiGrid> {
  LevelAiGrid::new(
    &Aabb {
      min: Vector3d::new(0.0, 0.0, 0.0),
      max: Vector3d::new(9.0, 1.0, 4.0),
    },
    1.0,
  )
}

#[test]
fn counts_each_side_with_a_cell_and_a_half_over_its_extent() -> XrfResult {
  let grid: LevelAiGrid = grid()?;

  assert_eq!((grid.get_row_length(), grid.get_column_length()), (5, 10));

  Ok(())
}

#[test]
fn unpacks_a_compressed_cell_to_its_corner() -> XrfResult {
  let grid: LevelAiGrid = grid()?;
  let packed: u32 = grid.compress_cell(2.0, 3.0)?;

  assert_eq!(packed, 2 * 5 + 3);
  assert_eq!(grid.unpack(packed), (2.0, 3.0));

  Ok(())
}

#[test]
fn places_a_point_by_the_compilers_rule_or_the_games() -> XrfResult {
  let grid: LevelAiGrid = grid()?;

  // Just short of a cell's border: the compiler's `EPS_L` tips it into the next cell, the game's lookup does not.
  assert_eq!(grid.compress_cell(0.4995, 0.0)?, 5);
  assert_eq!(grid.find_cell(0.4995, 0.0), (0, 0));
  // Off the grid the game's lookup names a cell outside it; the compiler refuses the point.
  assert_eq!(grid.find_cell(-3.0, 9.0), (-3, 9));
  assert!(grid.compress_cell(-3.0, 0.0).is_err());
  assert!(grid.compress_cell(0.0, 9.0).is_err());

  Ok(())
}

#[test]
fn refuses_a_grid_its_nodes_cannot_unpack_from() {
  let bounds: Aabb = Aabb {
    min: Vector3d::new(-5.0, -10.0, -3.0),
    max: Vector3d::new(5.0, 10.0, 3.0),
  };
  let inverted: Aabb = Aabb {
    min: bounds.max,
    max: bounds.min,
  };
  let unplaced: Aabb = Aabb {
    min: Vector3d::new(f32::NAN, -10.0, -3.0),
    ..bounds
  };

  assert!(LevelAiGrid::new(&bounds, 0.7).is_ok());

  for size in [0.0, -0.7, f32::NAN, f32::INFINITY] {
    assert!(LevelAiGrid::new(&bounds, size).is_err(), "a cell size of {size}");
  }

  assert!(LevelAiGrid::new(&inverted, 0.7).is_err(), "a box turned inside out");
  assert!(LevelAiGrid::new(&unplaced, 0.7).is_err(), "a box of no corner");
}
