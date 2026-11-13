use xrf_math::{Aabb, EPS_L};

/// The grid of square cells a `level.ai` packs each node's cell over: cells counted from a box's lowest corner, x in
/// rows of the cells along z (`CNodePositionCompressor`, `xrAI/compiler.h:170`, and `vertex_position`,
/// `compiler_load.cpp:8`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AiGrid {
  min_x: f32,
  min_z: f32,
  cell_size: f32,
  /// Cells along z, `iFloor((max.z - min.z) / size + EPS_L + 1.5)`.
  row_length: u32,
}

impl AiGrid {
  pub(crate) fn new(bounds: &Aabb, cell_size: f32) -> Self {
    Self {
      min_x: bounds.min.x,
      min_z: bounds.min.z,
      cell_size,
      row_length: ((bounds.max.z - bounds.min.z) / cell_size + EPS_L + 1.5).floor() as u32,
    }
  }

  /// The packed cell a point falls in: each axis `iFloor((value - min) * (1 / size) + EPS_L + 0.5)`.
  pub(crate) fn pack(&self, x: f32, z: f32) -> u32 {
    let reciprocal: f32 = 1.0 / self.cell_size;
    let axis = |value: f32, minimum: f32| ((value - minimum) * reciprocal + EPS_L + 0.5).floor() as i64;

    (axis(x, self.min_x) * i64::from(self.row_length) + axis(z, self.min_z)) as u32
  }

  /// A packed cell's corner position, x and z.
  pub(crate) fn unpack(&self, packed: u32) -> (f32, f32) {
    (
      (packed / self.row_length) as f32 * self.cell_size + self.min_x,
      (packed % self.row_length) as f32 * self.cell_size + self.min_z,
    )
  }
}
