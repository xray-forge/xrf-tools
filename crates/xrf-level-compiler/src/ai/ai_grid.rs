use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, EPS_L};

/// The grid of square cells a `level.ai` packs each node's cell over: cells counted from a box's lowest corner, x in
/// rows of the cells along z (`CNodePositionCompressor`, `xrAI/compiler.h`, and `vertex_position`,
/// `xrAI/compiler_load.cpp`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AiGrid {
  min_x: f32,
  min_z: f32,
  cell_size: f32,
  /// Cells along z, `iFloor((max.z - min.z) / size + EPS_L + 1.5)`.
  row_length: u32,
}

impl AiGrid {
  /// # Errors
  ///
  /// Returns an error when the cell size is not finite and positive, or the box spans no whole row of cells, either of
  /// which xrAI would pack into cells that unpack nowhere.
  pub(crate) fn new(bounds: &Aabb, cell_size: f32) -> XrfResult<Self> {
    if !cell_size.is_finite() || cell_size <= 0.0 {
      return Err(XrfError::new_invalid_error(format!(
        "AI map cell size {cell_size} is not a positive size"
      )));
    }

    let row_length: f32 = ((bounds.max.z - bounds.min.z) / cell_size + EPS_L + 1.5).floor();

    if !(1.0..=u32::MAX as f32).contains(&row_length) || !bounds.min.x.is_finite() {
      return Err(XrfError::new_invalid_error(format!(
        "AI map box from {:?} to {:?} spans no row of {cell_size} metre cells",
        bounds.min, bounds.max
      )));
    }

    Ok(Self {
      min_x: bounds.min.x,
      min_z: bounds.min.z,
      cell_size,
      row_length: row_length as u32,
    })
  }

  /// The packed cell a point falls in: each axis `iFloor((value - min) * (1 / size) + EPS_L + 0.5)`.
  ///
  /// # Errors
  ///
  /// Returns an error when the point lies before the box or its cell is past what 32 bits number, which xrAI would
  /// wrap into another cell.
  pub(crate) fn pack(&self, x: f32, z: f32) -> XrfResult<u32> {
    let reciprocal: f32 = 1.0 / self.cell_size;
    let axis = |value: f32, minimum: f32| ((value - minimum) * reciprocal + EPS_L + 0.5).floor() as i64;
    let (column, row) = (axis(x, self.min_x), axis(z, self.min_z));
    let outside = || XrfError::new_invalid_error(format!("AI map point at x {x}, z {z} falls outside the map's cells"));

    if column < 0 || !(0..i64::from(self.row_length)).contains(&row) {
      return Err(outside());
    }

    u32::try_from(column * i64::from(self.row_length) + row).map_err(|_| outside())
  }

  /// A packed cell's corner position, x and z.
  pub(crate) fn unpack(&self, packed: u32) -> (f32, f32) {
    (
      (packed / self.row_length) as f32 * self.cell_size + self.min_x,
      (packed % self.row_length) as f32 * self.cell_size + self.min_z,
    )
  }
}
