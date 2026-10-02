use std::collections::BTreeSet;

use crate::scene::level::shadow_tile::ShadowTile;

/// Hands out squares of an atlas whose sides are powers of two, as a buddy allocator does: a square is cut into four
/// quarters to make a smaller one, and four free quarters join back into their parent.
pub struct ShadowTileAllocator {
  size: u32,
  min: u32,
  /// The free squares of each side, by `log2(side)`, ordered so the same requests take the same squares.
  free: Vec<BTreeSet<(u32, u32)>>,
  used: u64,
}

impl ShadowTileAllocator {
  /// An atlas `size` texels across, cut no finer than `min`; both powers of two.
  pub fn new(size: u32, min: u32) -> Self {
    let mut free: Vec<BTreeSet<(u32, u32)>> = vec![BTreeSet::new(); size.ilog2() as usize + 1];

    free[size.ilog2() as usize].insert((0, 0));

    Self {
      size,
      min,
      free,
      used: 0,
    }
  }

  /// Texels handed out.
  pub fn get_used(&self) -> u64 {
    self.used
  }

  /// A free square of a side, rounded up to a power of two no less than the least, or none where there is no room.
  pub fn allocate(&mut self, size: u32) -> Option<ShadowTile> {
    let side: u32 = size.max(self.min).next_power_of_two();

    if side > self.size {
      return None;
    }

    let level: usize = side.ilog2() as usize;
    // The smallest free square at least as large, cut down to the side asked.
    let from: usize = (level..self.free.len()).find(|it| !self.free[*it].is_empty())?;
    let (x, y) = self.free[from].pop_first()?;

    for parent in (level + 1..=from).rev() {
      let half: u32 = 1 << (parent - 1);

      self.free[parent - 1].insert((x + half, y));
      self.free[parent - 1].insert((x, y + half));
      // The first quarter is kept, and cut further down where it must be.
      self.free[parent - 1].insert((x + half, y + half));
    }

    self.used += (side as u64) * (side as u64);

    Some(ShadowTile { x, y, size: side })
  }

  /// Gives a square back, joining it with its three free siblings into their parent as far up as they go.
  pub fn release(&mut self, tile: ShadowTile) {
    let (mut x, mut y, mut side) = (tile.x, tile.y, tile.size);

    self.used = self.used.saturating_sub((side as u64) * (side as u64));

    while side < self.size {
      let parent: u32 = side * 2;
      let (px, py) = (x - x % parent, y - y % parent);
      let siblings: [(u32, u32); 4] = [(px, py), (px + side, py), (px, py + side), (px + side, py + side)];
      let level: usize = side.ilog2() as usize;
      let are_free: bool = siblings
        .iter()
        .filter(|it| **it != (x, y))
        .all(|it| self.free[level].contains(it));

      if !are_free {
        break;
      }

      for sibling in siblings {
        self.free[level].remove(&sibling);
      }

      (x, y, side) = (px, py, parent);
    }

    self.free[side.ilog2() as usize].insert((x, y));
  }
}
