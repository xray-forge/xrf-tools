use xrf_math::{EPS_L, Vector3d};

/// The grid `VPack` finds already packed vertices through, as xrLC's welds (`CBuild::PreOptimize`,
/// `CDB::CollectorPacked`) and the SDK's exports (`CSkeletonCollectorPacked`, `CObjectOGFCollectorPacked`) lay it out:
/// `cells + 1` cells along each axis over a box.
///
/// A packed vertex is filed under its own cell and under the cells one reach above it on each axis ([`Self::get_reach`],
/// half a cell, at most `EPS_L`), and a point looks in its own cell only, in the order the vertices were filed. So the
/// grid decides which near vertices a point can meet, and is kept as the engine lays it out. How a point falls in a cell
/// is each caller's own: the arithmetic, and what a point outside the box comes to, differ between the engine's copies.
#[derive(Clone, Debug)]
pub struct PackGrid {
  /// The vertices filed under each cell, in the order they were filed.
  cells: Vec<Vec<u32>>,
  /// Cells along each axis past the first.
  counts: [usize; 3],
}

impl PackGrid {
  /// A grid of `counts` cells past the first along each axis.
  pub fn new(counts: [usize; 3]) -> Self {
    Self {
      cells: vec![Vec::new(); counts.iter().map(|count| count + 1).product()],
      counts,
    }
  }

  /// Cells along each axis past the first.
  pub fn get_counts(&self) -> [usize; 3] {
    self.counts
  }

  /// How far above itself a vertex is filed on each axis, over a box of `extent`: half a cell, at most `EPS_L`.
  pub fn get_reach(&self, extent: &Vector3d) -> Vector3d {
    let [x, y, z] = self.counts;
    let reach = |extent: f32, count: usize| (extent / count as f32 / 2.0).min(EPS_L);

    Vector3d::new(reach(extent.x, x), reach(extent.y, y), reach(extent.z, z))
  }

  /// The vertices filed under `cell`, in the order they were filed.
  pub fn get_filed(&self, cell: [usize; 3]) -> &[u32] {
    &self.cells[self.get_index(cell)]
  }

  /// Files `vertex` under `cell`, and under each cell the one a reach above it, `above`, moves into: along each axis
  /// it crosses on, then each pair, then all three, in `VPack`'s order.
  pub fn file(&mut self, vertex: u32, cell: [usize; 3], above: [usize; 3]) {
    let [x, y, z] = cell;
    let [x_above, y_above, z_above] = above;
    let home: usize = self.get_index(cell);

    self.cells[home].push(vertex);

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

        self.cells[filed].push(vertex);
      }
    }
  }

  fn get_index(&self, [x, y, z]: [usize; 3]) -> usize {
    let [_, count_y, count_z] = self.counts;

    (x * (count_y + 1) + y) * (count_z + 1) + z
  }
}
