use std::ops::Range;

use xrf_error::{XrfError, XrfResult};
use xrf_geometry::Box2Pick;
use xrf_level::{LevelAiGrid, LevelAiHeader, LevelAiNode, LevelAiNodes};
use xrf_math::{Aabb, AiNodePosition, Vector3d};

/// A level's AI map, `CLevelGraph`: its nodes in file order, sorted by packed cell, with the lookups that place points
/// on nodes.
pub(crate) struct SpawnAiMap {
  header: LevelAiHeader,
  grid: LevelAiGrid,
  nodes: Vec<LevelAiNode>,
}

impl SpawnAiMap {
  /// How many cells from the grid's corner a position may lie for [`Self::find_nearest_node`] to search it ring by
  /// ring: far enough for any map, near enough that ring arithmetic cannot overflow.
  const RING_REACH: u64 = 1 << 40;

  /// # Errors
  ///
  /// Returns an error when the header's grid does not unpack ([`LevelAiHeader::get_grid`]) or its node count differs
  /// from the nodes held.
  pub(crate) fn new(header: LevelAiHeader, nodes: &LevelAiNodes) -> XrfResult<Self> {
    let grid: LevelAiGrid = header.get_grid()?;
    let nodes: Vec<LevelAiNode> = nodes.iter_nodes().collect();

    if nodes.len() != header.count as usize {
      return Err(XrfError::new_invalid_error(format!(
        "Level AI map header counts {} nodes but holds {}",
        header.count,
        nodes.len()
      )));
    }

    Ok(Self { header, grid, nodes })
  }

  pub(crate) fn get_header(&self) -> &LevelAiHeader {
    &self.header
  }

  pub(crate) fn get_node_count(&self) -> usize {
    self.nodes.len()
  }

  /// The nodes a node links to, skipping `u24::MAX` and links past the map.
  pub(crate) fn get_neighbours(&self, node: u32) -> impl Iterator<Item = u32> + '_ {
    self.nodes[node as usize].get_neighbours(self.nodes.len() as u32)
  }

  /// `vertex_plane_y`: the height at `x`, `z` of the node's plane through [`Self::get_node_position`].
  pub(crate) fn get_plane_y(&self, node: u32, x: f32, z: f32) -> f32 {
    let point: Vector3d<f32> = self.get_node_position(node);
    let normal: Vector3d<f32> = self.nodes[node as usize].get_normal();

    point.y - (normal.x * (x - point.x) + normal.z * (z - point.z)) / normal.y
  }

  /// `valid_vertex_position`: whether the position lies over the map's grid, its box grown by half a cell.
  pub(crate) fn is_valid_position(&self, position: &Vector3d<f32>) -> bool {
    let (min, max) = (&self.header.bounds.min, &self.header.bounds.max);
    let half: f32 = self.header.size * 0.5;

    if position.x < min.x - half || position.x > max.x + half || position.z < min.z - half || position.z > max.z + half
    {
      return false;
    }

    let (x, z) = self.grid.find_cell(position.x, position.z);

    z < self.get_row_length() && x < i64::from(self.grid.get_column_length())
  }

  /// `CLevelGraph::vertex_id(position)`: the node of the position's cell nearest at or under it, else nearest over it;
  /// `None` for an empty cell.
  ///
  /// The caller checks the position lies over the grid.
  pub(crate) fn find_node(&self, position: &Vector3d<f32>) -> Option<u32> {
    let packed: u32 = self.get_packed_xz(position.x, position.z);
    let first: usize = self.nodes.partition_point(|node| node.packed_xz < packed);

    if self.nodes.get(first)?.packed_xz != packed {
      return None;
    }

    let mut best: usize = first;
    let mut best_y: f32 = self.get_plane_y(first as u32, position.x, position.z);

    for index in first + 1..self.nodes.len() {
      if self.nodes[index].packed_xz != packed {
        break;
      }

      let y: f32 = self.get_plane_y(index as u32, position.x, position.z);
      let is_nearer: bool = if best_y <= position.y {
        y <= position.y && position.y - y < position.y - best_y
      } else {
        y <= position.y || y - position.y < best_y - position.y
      };

      if is_nearer {
        best = index;
        best_y = y;
      }
    }

    Some(best as u32)
  }

  /// `inside(vertex, position)`: whether the position lies over the grid in the node's cell.
  pub(crate) fn is_inside(&self, node: u32, position: &Vector3d<f32>) -> bool {
    self.is_valid_position(position)
      && self.nodes[node as usize].packed_xz == self.get_packed_xz(position.x, position.z)
  }

  /// `CLevelGraph::vertex(u32(-1), position)`: [`Self::find_node`], else [`Self::find_nearest_node`] for an empty
  /// cell or a position off the grid.
  pub(crate) fn place(&self, position: &Vector3d<f32>) -> u32 {
    self
      .is_valid_position(position)
      .then(|| self.find_node(position))
      .flatten()
      .unwrap_or_else(|| self.find_nearest_node(position))
  }

  /// `CLevelGraph::vertex(position)`, the slow search: the node whose cell contour comes nearest the position, the
  /// lowest-numbered of equals, which is the first the engine's walk over every node keeps.
  ///
  /// Rather than walk every node, the search measures the cells in rings around the position's cell and stops at the
  /// first ring whose cells all lie farther across the ground than the best contour distance so far: the floor
  /// distance never exceeds the contour distance, so no node past it can come nearer. The engine's walk skips a node
  /// by the same floor and margin, which keeps rounding from passing over an equal; both find the same node.
  pub(crate) fn find_nearest_node(&self, position: &Vector3d<f32>) -> u32 {
    let (columns, rows): (i64, i64) = (i64::from(self.grid.get_column_length()), self.get_row_length());
    let (centre_x, centre_z) = self.grid.find_cell(position.x, position.z);

    // A position past the cells the rings count in, infinite included, is measured node by node.
    if centre_x.unsigned_abs() > Self::RING_REACH || centre_z.unsigned_abs() > Self::RING_REACH {
      return self.find_nearest_node_by_scan(position);
    }

    let half: f64 = f64::from(self.header.size) / 2.0;
    let across = |from: f32, to: f32| (f64::from(to - from).abs() - half).max(0.0);
    let reach = |distance: f32| f64::from(distance) * 1.001 + 0.01;
    let (mut nearest, mut nearest_distance): (u32, f32) = (0, f32::MAX);
    // The first ring holding a cell of the grid, for a position off it.
    let first: i64 = [-centre_x, centre_x - (columns - 1), -centre_z, centre_z - (rows - 1)]
      .into_iter()
      .fold(0, i64::max);

    for radius in first.. {
      for node in self.get_ring_nodes(centre_x, centre_z, radius) {
        let (x, z) = self.get_node_xz(node);

        if across(x, position.x).powi(2) + across(z, position.z).powi(2) > reach(nearest_distance) {
          continue;
        }

        let distance: f32 = self.get_contour_distance(node, position);

        if distance < nearest_distance || (distance == nearest_distance && node < nearest) {
          nearest = node;
          nearest_distance = distance;
        }
      }

      let is_grid_covered: bool = centre_x - radius <= 0
        && centre_x + radius >= columns - 1
        && centre_z - radius <= 0
        && centre_z + radius >= rows - 1;

      // Every cell past this ring lies a ring farther along x or along z, and so at least as far as the next ring's
      // nearer side along that axis.
      let next: i64 = radius + 1;
      let (low_x, low_z) = self.grid.get_cell_position(centre_x - next, centre_z - next);
      let (high_x, high_z) = self.grid.get_cell_position(centre_x + next, centre_z + next);
      let beyond: f64 = across(low_x, position.x)
        .min(across(high_x, position.x))
        .powi(2)
        .min(across(low_z, position.z).min(across(high_z, position.z)).powi(2));

      if is_grid_covered || beyond > reach(nearest_distance) {
        break;
      }
    }

    nearest
  }

  /// [`Self::find_nearest_node`] as the engine walks it, every node in turn: the first node whose cell contour comes
  /// nearest. A node whose cell is clearly farther across the ground than the best so far is skipped unmeasured; the
  /// margin keeps rounding from skipping an equal.
  pub(crate) fn find_nearest_node_by_scan(&self, position: &Vector3d<f32>) -> u32 {
    let half: f64 = f64::from(self.header.size) / 2.0;
    let across = |from: f32, to: f32| (f64::from(to - from).abs() - half).max(0.0);
    let (mut nearest, mut nearest_distance): (u32, f32) = (0, f32::MAX);

    for node in 0..self.nodes.len() as u32 {
      let (x, z) = self.get_node_xz(node);
      let floor: f64 = across(x, position.x).powi(2) + across(z, position.z).powi(2);

      if floor > f64::from(nearest_distance) * 1.001 + 0.01 {
        continue;
      }

      let distance: f32 = self.get_contour_distance(node, position);

      if distance < nearest_distance {
        nearest = node;
        nearest_distance = distance;
      }
    }

    nearest
  }

  /// The nodes of the cells on the square ring `radius` cells around the cell at `column`, `row`, which may lie off the
  /// grid; the ring of radius 0 is that cell alone.
  fn get_ring_nodes(&self, column: i64, row: i64, radius: i64) -> impl Iterator<Item = u32> + '_ {
    let columns: i64 = i64::from(self.grid.get_column_length());
    let edges: Vec<i64> = if radius == 0 {
      vec![column]
    } else {
      vec![column - radius, column + radius]
    };
    let sides: Vec<Range<u32>> = ((column - radius + 1).max(0)..=(column + radius - 1).min(columns - 1))
      .flat_map(|side| {
        [
          self.get_row_nodes(side, row - radius, row - radius),
          self.get_row_nodes(side, row + radius, row + radius),
        ]
      })
      .collect();

    edges
      .into_iter()
      .map(move |edge| self.get_row_nodes(edge, row - radius, row + radius))
      .chain(sides)
      .flatten()
  }

  /// The nodes of the cells in `column` from row `from` to row `to`, both included, each clipped to the grid: with
  /// nodes sorted by packed cell, those of a run of cells along z lie together.
  fn get_row_nodes(&self, column: i64, from: i64, to: i64) -> Range<u32> {
    let rows: i64 = self.get_row_length();
    let (from, to) = (from.max(0), to.min(rows - 1));

    if !(0..i64::from(self.grid.get_column_length())).contains(&column) || from > to {
      return 0..0;
    }

    let packed = |row: i64| (column * rows + row) as u32;
    let (start, end) = (packed(from), packed(to) + 1);

    self.nodes.partition_point(|node| node.packed_xz < start) as u32
      ..self.nodes.partition_point(|node| node.packed_xz < end) as u32
  }

  /// `CLevelGraph::distance(position, vertex)`: the squared distance to the nearest edge of the node's cell contour
  /// projected onto its plane.
  ///
  /// The plane offset and corner heights are computed in `f64` and rounded once, as the 2009 build's x87 code keeps
  /// them in registers. In single precision the dot products lose tens of ulps, enough to pick the other of two cells
  /// sharing an edge.
  fn get_contour_distance(&self, node: u32, position: &Vector3d<f32>) -> f32 {
    let centre: Vector3d<f64> = Self::widen(&self.get_node_position(node));
    let normal: Vector3d<f64> = Self::widen(&self.nodes[node as usize].get_normal());
    let offset: f64 = f64::from((-(normal.x * centre.x + normal.y * centre.y + normal.z * centre.z)) as f32);
    let half: f32 = self.header.size / 2.0;
    let corner = |dx: f32, dz: f32| {
      // `v.set(position.x ± st, ...)` stores each corner's x and z in single precision before projecting it.
      let (x, z) = ((centre.x as f32) + dx, (centre.z as f32) + dz);
      let classify: f64 = normal.x * f64::from(x) + normal.y * centre.y + normal.z * f64::from(z) + offset;

      Vector3d::new(x, (centre.y - classify / normal.y) as f32, z)
    };
    let corners: [Vector3d<f32>; 4] = [
      corner(-half, -half),
      corner(half, -half),
      corner(half, half),
      corner(-half, half),
    ];

    (0..4)
      .map(|index| Self::get_segment_distance(position, &corners[index], &corners[(index + 1) % 4]))
      .fold(f32::MAX, f32::min)
  }

  fn widen(vector: &Vector3d<f32>) -> Vector3d<f64> {
    Vector3d {
      x: f64::from(vector.x),
      y: f64::from(vector.y),
      z: f64::from(vector.z),
    }
  }

  /// The squared distance from a point to a segment, `CLevelGraph::distance(position, point0, point1)`.
  fn get_segment_distance(position: &Vector3d<f32>, start: &Vector3d<f32>, end: &Vector3d<f32>) -> f32 {
    let squared = |a: &Vector3d<f32>, b: &Vector3d<f32>| {
      let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);

      dx * dx + dy * dy + dz * dz
    };
    let (cx, cy, cz) = (position.x - start.x, position.y - start.y, position.z - start.z);
    let (vx, vy, vz) = (end.x - start.x, end.y - start.y, end.z - start.z);
    let length: f32 = (vx * vx + vy * vy + vz * vz).sqrt();
    let (vx, vy, vz) = (vx / length, vy / length, vz / length);
    let t: f32 = vx * cx + vy * cy + vz * cz;

    if t <= 0.0 {
      squared(position, start)
    } else if t >= length {
      squared(position, end)
    } else {
      squared(
        position,
        &Vector3d::new(start.x + vx * t, start.y + vy * t, start.z + vz * t),
      )
    }
  }

  /// A node's cell along x and z, `unpack_xz` into whole cells.
  pub(crate) fn get_node_cell(&self, node: u32) -> (i64, i64) {
    let packed: i64 = i64::from(self.nodes[node as usize].packed_xz);

    (packed / self.get_row_length(), packed % self.get_row_length())
  }

  /// A node's position, `vertex_position`: its cell centre at its packed height, unpacked as the shipped game does
  /// ([`AiNodePosition::unpack_height_as_built`]).
  ///
  /// Every lookup here takes a node's height from it, as the engine's do.
  pub(crate) fn get_node_position(&self, node: u32) -> Vector3d<f32> {
    let (x, z) = self.get_node_xz(node);
    let height: f32 = AiNodePosition::unpack_height_as_built(
      self.nodes[node as usize].packed_y,
      self.header.bounds.min.y,
      self.header.size_y,
    );

    Vector3d::new(x, height, z)
  }

  /// Where a node's cell centre stands across the ground, `unpack_xz`.
  pub(crate) fn get_node_xz(&self, node: u32) -> (f32, f32) {
    self.grid.unpack(self.nodes[node as usize].packed_xz)
  }

  /// `CLevelGraph::check_position_in_direction`: the node at `to` that a flat straight walk from node `start`, at
  /// `from`, reaches; `None` where it gets stuck or `to` lies off the grid.
  ///
  /// Each step takes the first link, never straight back, whose cell the line crosses (`Fbox2::pick_exact`) and whose
  /// centre comes no farther from `to`. A walk circling cells equally far from `to`, which never returns in the
  /// engine, gives up after as many steps as there are nodes.
  pub(crate) fn find_straight_walk(&self, start: u32, from: (f32, f32), to: (f32, f32)) -> Option<u32> {
    let target: u32 = self.get_packed_xz(to.0, to.1);

    if self.nodes[start as usize].packed_xz == target {
      return Some(start);
    }

    if !self.is_valid_position(&Vector3d::new(to.0, 0.0, to.1)) {
      return None;
    }

    let half: f32 = self.header.size * 0.5;
    let direction: (f32, f32) = (to.0 - from.0, to.1 - from.1);
    let (start_x, start_z) = self.get_node_xz(start);
    let mut squared_distance: f32 = (start_x - to.0) * (start_x - to.0) + (start_z - to.1) * (start_z - to.1);
    let (mut current, mut previous): (u32, u32) = (start, u32::MAX);

    for _ in 0..self.nodes.len() {
      let mut step: Option<(u32, f32)> = None;

      for next in self.get_neighbours(current) {
        if next == previous {
          continue;
        }

        let (x, z) = self.get_node_xz(next);
        let (min, max) = ((x - half, z - half), (x + half, z + half));

        if !Box2Pick::is_crossing(min, max, from, direction) {
          continue;
        }

        if self.nodes[next as usize].packed_xz == target {
          return Some(next);
        }

        #[expect(
          clippy::manual_midpoint,
          reason = "the engine sums in single precision, where `f32::midpoint` sums wider"
        )]
        let middle: (f32, f32) = ((min.0 + max.0) * 0.5, (min.1 + max.1) * 0.5);
        let distance: f32 = (middle.0 - to.0) * (middle.0 - to.0) + (middle.1 - to.1) * (middle.1 - to.1);

        if distance > squared_distance {
          continue;
        }

        step = Some((next, distance));
        break;
      }

      let (next, distance) = step?;

      squared_distance = distance;
      previous = current;
      current = next;
    }

    None
  }

  /// The nodes, in node order, whose cells lie across the ground within `bounds` grown by a cell.
  pub(crate) fn get_nodes_within(&self, bounds: &Aabb) -> Vec<u32> {
    let size: f32 = self.grid.get_cell_size();
    let (min_x, min_z) = self.grid.find_cell(bounds.min.x - size, bounds.min.z - size);
    let (max_x, max_z) = self.grid.find_cell(bounds.max.x + size, bounds.max.z + size);

    (min_x.max(0)..=max_x.min(i64::from(self.grid.get_column_length()) - 1))
      .flat_map(|x| self.get_row_nodes(x, min_z, max_z))
      .collect()
  }

  /// The packed cell a point across the ground falls in.
  fn get_packed_xz(&self, x: f32, z: f32) -> u32 {
    let (cell_x, cell_z) = self.grid.find_cell(x, z);

    (cell_x * self.get_row_length() + cell_z) as u32
  }

  /// Cells along z, the stride of x in a packed cell.
  fn get_row_length(&self) -> i64 {
    i64::from(self.grid.get_row_length())
  }
}
