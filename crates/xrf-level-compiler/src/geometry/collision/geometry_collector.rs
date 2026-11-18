use xrf_geometry::PackGrid;
use xrf_math::{Aabb, EPS_L, Vector3d};

/// Points packed into vertices as the collision forms are, `CDB::CollectorPacked` (`xrCDB/xrCDB_Collector.cpp`): a
/// point takes a vertex already packed within `EPS_L` on every axis, strictly, found through a [`PackGrid`] of
/// [`Self::CELLS`] over a box, or becomes one.
///
/// A point outside the box is clamped into the grid as `VPack` clamps it: its cell index is an unsigned one, so a point
/// below the box lands in the last cell of that axis, not the first.
#[derive(Clone, Debug)]
pub(crate) struct GeometryCollector {
  vertices: Vec<Vector3d>,
  grid: PackGrid,
  minimum: Vector3d,
  size: Vector3d,
  reach: Vector3d,
}

impl GeometryCollector {
  /// Cells of the grid along each axis, past the first (`clpMX`, `clpMY`, `clpMZ`).
  pub(crate) const CELLS: [usize; 3] = [24, 16, 24];

  /// A collector over `bounds`.
  pub(crate) fn new(bounds: &Aabb) -> Self {
    let size: Vector3d = &bounds.max - &bounds.min;
    let grid: PackGrid = PackGrid::new(Self::CELLS);

    Self {
      vertices: Vec::new(),
      minimum: bounds.min,
      reach: grid.get_reach(&size),
      grid,
      size,
    }
  }

  /// The vertex `point` packs into, `VPack`.
  pub(crate) fn pack(&mut self, point: &Vector3d) -> u32 {
    let cell: [usize; 3] = self.get_cell(point);

    if let Some(found) = self
      .grid
      .get_filed(cell)
      .iter()
      .copied()
      .find(|vertex| self.vertices[*vertex as usize].similar(point, EPS_L))
    {
      return found;
    }

    let index: u32 = self.vertices.len() as u32;
    let above: [usize; 3] = self.get_cell(&(point + &self.reach));

    self.vertices.push(*point);
    self.grid.file(index, cell, above);

    index
  }

  /// The vertices packed, taken.
  pub(crate) fn into_vertices(self) -> Vec<Vector3d> {
    self.vertices
  }

  /// The cell `point` falls in: `iFloor((p - min) / size * cells)` on each axis, its unsigned clamp included.
  fn get_cell(&self, point: &Vector3d) -> [usize; 3] {
    let axis = |value: f32, minimum: f32, size: f32, cells: usize| -> usize {
      let cell: f32 = ((value - minimum) / size * cells as f32).floor();

      // A negative index read as unsigned is past every cell, and clamps to the last.
      if cell < 0.0 || cell.is_nan() {
        cells
      } else {
        (cell as usize).min(cells)
      }
    };
    let [cells_x, cells_y, cells_z] = Self::CELLS;

    [
      axis(point.x, self.minimum.x, self.size.x, cells_x),
      axis(point.y, self.minimum.y, self.size.y, cells_y),
      axis(point.z, self.minimum.z, self.size.z, cells_z),
    ]
  }
}
