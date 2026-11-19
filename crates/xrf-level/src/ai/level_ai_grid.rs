use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, EPS_L};

/// The grid of square cells a `level.ai` packs node positions over (`CLevelGraph::Initialize`, `vertex_position`;
/// xrAI's `CNodePositionCompressor`).
///
/// Cells count from the box's lowest corner; a packed cell is its row along x times the row length plus its cell along
/// z. The compiler ([`Self::compress_cell`]) and the game ([`Self::find_cell`]) place a point by different rules, kept
/// side by side here so neither drifts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelAiGrid {
  min_x: f32,
  min_z: f32,
  cell_size: f32,
  row_length: u32,
  column_length: u32,
}

impl LevelAiGrid {
  /// The grid of `cell_size` cells over `bounds`.
  ///
  /// # Errors
  ///
  /// Returns an error for a cell size that is not finite and positive, or a box that spans no cell along either side:
  /// the engine divides every packed cell by the row length.
  pub fn new(bounds: &Aabb, cell_size: f32) -> XrfResult<Self> {
    if !cell_size.is_finite() || cell_size <= 0.0 {
      return Err(XrfError::new_invalid_error(format!(
        "AI map cell size {cell_size} is not a positive size"
      )));
    }

    let grid: Self = Self::covering(bounds, cell_size);

    if grid.row_length == 0 || grid.column_length == 0 || !grid.min_x.is_finite() || !grid.min_z.is_finite() {
      return Err(XrfError::new_invalid_error(format!(
        "AI map box from {:?} to {:?} spans no {cell_size} metre cell",
        bounds.min, bounds.max
      )));
    }

    Ok(grid)
  }

  /// The grid over `bounds` as the engine computes it, unchecked: a side spanning no cell counts none.
  pub(crate) fn covering(bounds: &Aabb, cell_size: f32) -> Self {
    Self {
      min_x: bounds.min.x,
      min_z: bounds.min.z,
      cell_size,
      row_length: Self::count_cells(bounds.min.z, bounds.max.z, cell_size),
      column_length: Self::count_cells(bounds.min.x, bounds.max.x, cell_size),
    }
  }

  pub fn get_cell_size(&self) -> f32 {
    self.cell_size
  }

  /// Cells along z, the stride a packed cell counts x in (`m_row_length`).
  pub fn get_row_length(&self) -> u32 {
    self.row_length
  }

  /// Cells along x (`m_column_length`).
  pub fn get_column_length(&self) -> u32 {
    self.column_length
  }

  /// The packed cell xrAI writes a point in (`CNodePositionCompressor`): each axis
  /// `iFloor((value - min) * (1 / size) + EPS_L + 0.5)`.
  ///
  /// # Errors
  ///
  /// Returns an error when the point lies before the box, past its row, or past a 32-bit cell number: xrAI would wrap
  /// each into another cell.
  pub fn compress_cell(&self, x: f32, z: f32) -> XrfResult<u32> {
    let reciprocal: f32 = 1.0 / self.cell_size;
    let axis = |value: f32, minimum: f32| ((value - minimum) * reciprocal + EPS_L + 0.5).floor() as i64;
    let (column, row) = (axis(x, self.min_x), axis(z, self.min_z));
    let outside = || XrfError::new_invalid_error(format!("AI map point at x {x}, z {z} falls outside the map's cells"));

    if column < 0 || !(0..i64::from(self.row_length)).contains(&row) {
      return Err(outside());
    }

    u32::try_from(column * i64::from(self.row_length) + row).map_err(|_| outside())
  }

  /// The cell the game looks a point up in (`CLevelGraph::vertex_id`), along x and z: each axis
  /// `iFloor((value - min) / size + 0.5)`, unbounded, so a point off the grid gives a cell outside it.
  pub fn find_cell(&self, x: f32, z: f32) -> (i64, i64) {
    (
      ((x - self.min_x) / self.cell_size + 0.5).floor() as i64,
      ((z - self.min_z) / self.cell_size + 0.5).floor() as i64,
    )
  }

  /// A packed cell's corner, along x and z (`vertex_position`). With no row, every cell unpacks into the first row.
  pub fn unpack(&self, packed: u32) -> (f32, f32) {
    let row_length: u32 = self.row_length.max(1);

    (
      (packed / row_length) as f32 * self.cell_size + self.min_x,
      (packed % row_length) as f32 * self.cell_size + self.min_z,
    )
  }

  /// `iFloor(extent / cell_size + EPS_L + 1.5f)`, as `CLevelGraph::Initialize` computes both grid sides.
  fn count_cells(minimum: f32, maximum: f32, cell_size: f32) -> u32 {
    ((maximum - minimum) / cell_size + EPS_L + 1.5).floor().max(0.0) as u32
  }
}
