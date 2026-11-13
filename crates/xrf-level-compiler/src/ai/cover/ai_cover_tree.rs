use xrf_math::{Aabb, Vector3d};

/// xrAI's quadtree of cover points (`CQuadTree`, `xrAI/quadtree_inline.h`), ported as it is so its query returns the
/// points in the engine's order: the mix of the other nodes sums their weights in that order. Its query also prunes as
/// the engine's does, which can leave out a point within reach; kept, since the engine's mix leaves it out too.
pub(crate) struct AiCoverTree {
  center: Vector3d,
  radius: f32,
  max_depth: u32,
  /// Each inner node's four children, 0 for none; the root is 1 once anything is inserted.
  nodes: Vec<[u32; 4]>,
  /// Each leaf's points by node index, in insertion order; the engine prepends, so a leaf is read last first.
  leaves: Vec<Vec<u32>>,
  root: u32,
}

/// What one query of the tree looks for: the points within `radius` of `position` across the ground, each point's
/// position by its node in `positions`.
struct AiCoverTreeSearch<'a> {
  position: &'a Vector3d,
  radius: f32,
  positions: &'a [Vector3d],
}

impl AiCoverTree {
  /// A tree over `bounds`, its leaves at least `min_cell_size` across (the patch size halved).
  pub(crate) fn new(bounds: &Aabb, min_cell_size: f32) -> Self {
    let radius: f32 = (bounds.max.x - bounds.min.x).max(bounds.max.z - bounds.min.z) * 0.5;
    let center: Vector3d = bounds.min.midpoint(&bounds.max);
    let depth: f32 = ((2.0 * radius / min_cell_size).ln() / 2.0f32.ln() + 0.5).floor();

    Self {
      center,
      radius,
      max_depth: depth.max(0.0) as u32,
      nodes: vec![[0; 4]],
      leaves: Vec::new(),
      root: 0,
    }
  }

  /// `neighbour_index`: the quarter of the cell around `center` that `position` falls in, moving `center` to its middle.
  fn get_quarter(position: &Vector3d, center: &mut Vector3d, distance: f32) -> usize {
    let quarter: usize = match (position.x <= center.x, position.z <= center.z) {
      (true, true) => 0,
      (true, false) => 1,
      (false, true) => 2,
      (false, false) => 3,
    };

    center.x += if quarter < 2 { -distance } else { distance };
    center.z += if quarter & 1 == 0 { -distance } else { distance };

    quarter
  }

  /// Adds the point of node `node` standing at `position`.
  pub(crate) fn insert(&mut self, node: u32, position: &Vector3d) {
    let mut center: Vector3d = self.center;
    let mut distance: f32 = self.radius;
    let mut slot: Option<(usize, usize)> = None;

    for depth in 0.. {
      let current: u32 = match slot {
        None => self.root,
        Some((parent, quarter)) => self.nodes[parent][quarter],
      };

      if depth == self.max_depth {
        let leaf: u32 = if current == 0 {
          self.leaves.push(Vec::new());

          let leaf: u32 = self.leaves.len() as u32;

          self.set(slot, leaf);
          leaf
        } else {
          current
        };

        self.leaves[leaf as usize - 1].push(node);

        return;
      }

      let current: u32 = if current == 0 {
        self.nodes.push([0; 4]);

        let created: u32 = (self.nodes.len() - 1) as u32;

        self.set(slot, created);
        created
      } else {
        current
      };

      distance *= 0.5;
      slot = Some((current as usize, Self::get_quarter(position, &mut center, distance)));
    }
  }

  fn set(&mut self, slot: Option<(usize, usize)>, value: u32) {
    match slot {
      None => self.root = value,
      Some((parent, quarter)) => self.nodes[parent][quarter] = value,
    }
  }

  /// The points within `radius` of `position` across the ground, in the order the engine's `nearest` lists them, in
  /// place of what `found` held. `positions` gives each node's position.
  pub(crate) fn find_nearest(&self, position: &Vector3d, radius: f32, positions: &[Vector3d], found: &mut Vec<u32>) {
    found.clear();
    self.visit(
      &AiCoverTreeSearch {
        position,
        radius,
        positions,
      },
      found,
      self.root,
      self.center,
      self.radius,
      0,
    );
  }

  /// One cell of `nearest` (`quadtree_inline.h:149`), its children walked as the engine walks them.
  fn visit(
    &self,
    search: &AiCoverTreeSearch<'_>,
    found: &mut Vec<u32>,
    node: u32,
    center: Vector3d,
    distance: f32,
    depth: u32,
  ) {
    let (position, radius): (&Vector3d, f32) = (search.position, search.radius);

    if node == 0 {
      return;
    }

    if depth == self.max_depth {
      let radius_squared: f32 = radius * radius;

      for point in self.leaves[node as usize - 1].iter().rev() {
        let at: &Vector3d = &search.positions[*point as usize];
        let (x, z) = (at.x - position.x, at.z - position.z);

        if x * x + z * z <= radius_squared {
          found.push(*point);
        }
      }

      return;
    }

    let distance: f32 = distance * 0.5;
    let mut next: Vector3d = center;
    let index: usize = Self::get_quarter(position, &mut next, distance);
    let children: [u32; 4] = self.nodes[node as usize];
    let child = |quarter: usize, x: f32, z: f32| {
      (
        children[quarter],
        Vector3d::new(center.x + x * distance, center.y, center.z + z * distance),
      )
    };
    let walk = |(child, at): (u32, Vector3d), found: &mut Vec<u32>| {
      self.visit(search, found, child, at, distance, depth + 1);
    };
    // The cell beside the one the point falls in across z, and across x, as the engine names them.
    let across_z = || {
      if index & 1 == 1 {
        child(
          if index == 1 { 0 } else { 2 },
          if index == 1 { -1.0 } else { 1.0 },
          -1.0,
        )
      } else {
        child(if index == 0 { 1 } else { 3 }, if index == 0 { -1.0 } else { 1.0 }, 1.0)
      }
    };
    let across_x = || {
      if index > 1 {
        child(
          if index == 2 { 0 } else { 1 },
          -1.0,
          if index == 2 { -1.0 } else { 1.0 },
        )
      } else {
        child(if index == 0 { 2 } else { 3 }, 1.0, if index == 0 { -1.0 } else { 1.0 })
      }
    };
    let is_z_ahead: bool = position.z > center.z;
    let is_x_ahead: bool = position.x > center.x;
    let wants_z = |index: usize| if is_z_ahead { index & 1 == 1 } else { index & 1 == 0 };
    let wants_x = |index: usize| if is_x_ahead { index > 1 } else { index < 2 };

    if (position.z - center.z).abs() < radius {
      if (position.x - center.x).abs() < radius {
        let (dz, dx) = (position.z - center.z, position.x - center.x);

        if dz * dz + dx * dx < radius * radius {
          walk(child(0, -1.0, -1.0), found);
          walk(child(1, -1.0, 1.0), found);
          walk(child(2, 1.0, -1.0), found);
          walk(child(3, 1.0, 1.0), found);

          return;
        }

        walk((children[index], next), found);

        if wants_z(index) {
          walk(across_z(), found);
        }

        if wants_x(index) {
          walk(across_x(), found);
        }

        return;
      }

      walk((children[index], next), found);

      if wants_z(index) {
        walk(across_z(), found);
      }

      return;
    }

    walk((children[index], next), found);

    if (position.x - center.x).abs() < radius && wants_x(index) {
      walk(across_x(), found);
    }
  }
}
