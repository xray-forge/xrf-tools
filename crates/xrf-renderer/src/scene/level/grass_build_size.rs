/// Bytes a cached tuft or a planted item takes: two vectors.
pub const GRASS_ITEM_BYTES: u64 = 32;

/// How large a build of the grass's ring and item lists is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrassBuildSize {
  /// Slots the ring holds.
  pub cells: u32,
  /// Tufts a cell holds at most.
  pub per_cell: u32,
  /// Rings of cells around the camera's.
  pub bands: u32,
  /// Items the lists hold.
  pub capacity: u32,
}

impl GrassBuildSize {
  /// What a reach and a slot's candidates want, within what one storage binding may hold: past it the planting plants
  /// what fits, dropping the rest. The lists are a power of two, so a setting dragged rebuilds them a handful of times.
  pub fn wanted(reach: u32, candidates: u32, binding_limit: u64) -> Self {
    let line: u32 = reach * 2 + 1;
    let cells: u32 = line * line;
    let per_cell: u64 = u64::from(candidates).min(binding_limit / (u64::from(cells.max(1)) * GRASS_ITEM_BYTES));
    let most: u64 = binding_limit / GRASS_ITEM_BYTES;

    Self {
      cells,
      per_cell: per_cell.max(1) as u32,
      bands: reach + 1,
      capacity: u64::from(cells * candidates).next_power_of_two().clamp(1, most) as u32,
    }
  }

  /// Whether a build of this size holds what another wants without more than twice its room, which gives memory back
  /// when a radius or density is brought back down.
  pub fn is_fitting(&self, wanted: &Self) -> bool {
    let is_outgrown: bool = wanted.capacity > self.capacity
      || wanted.cells > self.cells
      || wanted.per_cell > self.per_cell
      || wanted.bands > self.bands;

    !is_outgrown && self.get_bytes() <= 2 * wanted.get_bytes()
  }

  /// Bytes its ring and lists take.
  fn get_bytes(&self) -> u64 {
    u64::from(self.capacity) * (GRASS_ITEM_BYTES * 2 + 4)
      + u64::from(self.cells) * u64::from(self.per_cell) * GRASS_ITEM_BYTES
  }
}
