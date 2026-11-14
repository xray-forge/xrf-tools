use xrf_math::{Aabb, EPS_L, Vector3d};

/// Points packed into vertices as the collision forms are, `CDB::CollectorPacked` (`xrCDB/xrCDB_Collector.cpp:281`): a
/// point takes a vertex already packed within `EPS_L` on every axis, strictly, found through a grid of
/// [`Self::CELLS`] over a box, or becomes one.
///
/// A packed vertex is filed under its cell and under the cells one reach above it on each axis (half a cell, at most
/// `EPS_L`), and a point looks in its own cell only, in the order the vertices were packed. A point outside the box is
/// clamped into the grid as `VPack` clamps it: its cell index is an unsigned one, so a point below the box lands in the
/// last cell of that axis, not the first.
#[derive(Clone, Debug)]
pub(crate) struct GeometryCollector {
  vertices: Vec<Vector3d>,
  cells: Vec<Vec<u32>>,
  minimum: Vector3d,
  size: Vector3d,
  reach: Vector3d,
}

impl GeometryCollector {
  /// Cells of the grid along each axis, past the first (`clpMX`, `clpMY`, `clpMZ`).
  pub(crate) const CELLS: [usize; 3] = [24, 16, 24];

  /// A collector over `bounds`.
  pub(crate) fn new(bounds: &Aabb) -> Self {
    let [cells_x, cells_y, cells_z] = Self::CELLS;
    let size: Vector3d = &bounds.max - &bounds.min;

    Self {
      vertices: Vec::new(),
      cells: vec![Vec::new(); (cells_x + 1) * (cells_y + 1) * (cells_z + 1)],
      minimum: bounds.min,
      reach: Vector3d::new(
        (size.x / cells_x as f32 / 2.0).min(EPS_L),
        (size.y / cells_y as f32 / 2.0).min(EPS_L),
        (size.z / cells_z as f32 / 2.0).min(EPS_L),
      ),
      size,
    }
  }

  /// The vertex `point` packs into, `VPack`.
  pub(crate) fn pack(&mut self, point: &Vector3d) -> u32 {
    let cell: [usize; 3] = self.get_cell(point);

    if let Some(found) = self.cells[self.get_index(cell)]
      .iter()
      .copied()
      .find(|vertex| Self::is_similar(&self.vertices[*vertex as usize], point))
    {
      return found;
    }

    let index: u32 = self.vertices.len() as u32;
    let [x, y, z] = cell;
    let [x_above, y_above, z_above] = self.get_cell(&(point + &self.reach));

    self.vertices.push(*point);

    let home: usize = self.get_index(cell);

    self.cells[home].push(index);

    for (is_x, is_y, is_z) in [
      (true, false, false),
      (false, true, false),
      (false, false, true),
      (true, true, false),
      (true, false, true),
      (false, true, true),
      (true, true, true),
    ] {
      if (!is_x || x_above != x) && (!is_y || y_above != y) && (!is_z || z_above != z) {
        let filed: usize = self.get_index([
          if is_x { x_above } else { x },
          if is_y { y_above } else { y },
          if is_z { z_above } else { z },
        ]);

        self.cells[filed].push(index);
      }
    }

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

  fn get_index(&self, [x, y, z]: [usize; 3]) -> usize {
    let [_, cells_y, cells_z] = Self::CELLS;

    (x * (cells_y + 1) + y) * (cells_z + 1) + z
  }

  /// `Fvector::similar` at its default, `EPS_L`: every component strictly within it.
  fn is_similar(first: &Vector3d, second: &Vector3d) -> bool {
    (first.x - second.x).abs() < EPS_L && (first.y - second.y).abs() < EPS_L && (first.z - second.z).abs() < EPS_L
  }
}
