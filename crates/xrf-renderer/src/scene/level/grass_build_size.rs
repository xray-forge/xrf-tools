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
  /// Bytes a cached tuft or a planted item takes: two vectors.
  pub const ITEM_BYTES: u64 = 32;

  /// What a reach and a slot's candidates want, within what one storage binding may hold: past it the planting plants
  /// what fits, dropping the rest. The lists are a power of two, so a setting dragged rebuilds them a handful of times.
  pub fn wanted(reach: u32, candidates: u32, binding_limit: u64) -> Self {
    // Counted in 64 bits: a wide reach at a dense spacing passes a 32-bit count before the binding limit bounds it.
    let line: u64 = u64::from(reach) * 2 + 1;
    let cells: u64 = line.saturating_mul(line);
    let per_cell: u64 = u64::from(candidates).min(binding_limit / cells.max(1).saturating_mul(Self::ITEM_BYTES));
    let most: u64 = (binding_limit / Self::ITEM_BYTES).min(u64::from(u32::MAX));

    Self {
      cells: cells.min(u64::from(u32::MAX)) as u32,
      per_cell: per_cell.max(1) as u32,
      bands: reach.saturating_add(1),
      capacity: cells
        .saturating_mul(u64::from(candidates))
        .checked_next_power_of_two()
        .map_or(most, |capacity| capacity.clamp(1, most)) as u32,
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
    u64::from(self.capacity) * (Self::ITEM_BYTES * 2 + 4)
      + u64::from(self.cells) * u64::from(self.per_cell) * Self::ITEM_BYTES
  }
}
