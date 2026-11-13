use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelAiHeader, LevelAiNode, LevelAiNodes};
use xrf_math::Vector3d;

use crate::ai::ai_cell_pick::AiCellPick;

/// A level's AI map as a root spawn build reads it, `CLevelGraph`: its nodes in their file order, sorted by packed
/// cell, with the lookups that place a graph point, an entity or a patrol point on a node.
pub struct SpawnAiMap {
  header: LevelAiHeader,
  nodes: Vec<LevelAiNode>,
  /// Cells along z, the stride a packed cell counts x in.
  row_length: i64,
  /// Cells along x.
  column_length: i64,
}

impl SpawnAiMap {
  pub fn new(header: LevelAiHeader, nodes: &LevelAiNodes) -> Self {
    Self {
      row_length: i64::from(header.get_row_length()),
      column_length: i64::from(header.get_column_length()),
      nodes: nodes.iter_nodes::<XRayByteOrder>().collect(),
      header,
    }
  }

  pub fn get_header(&self) -> &LevelAiHeader {
    &self.header
  }

  pub fn get_node_count(&self) -> usize {
    self.nodes.len()
  }

  /// The nodes a node's links name; a link to no node, `u24::MAX` or past the map, is none.
  pub fn get_neighbours(&self, node: u32) -> impl Iterator<Item = u32> + '_ {
    self.nodes[node as usize].get_neighbours(self.nodes.len() as u32)
  }

  /// `vertex_plane_y`: the height of the node's plane at `x`, `z`.
  pub fn get_plane_y(&self, node: u32, x: f32, z: f32) -> f32 {
    let vertex: &LevelAiNode = &self.nodes[node as usize];
    let (point, normal): (Vector3d<f32>, Vector3d<f32>) = (self.header.get_node_position(vertex), vertex.get_normal());

    point.y - (normal.x * (x - point.x) + normal.z * (z - point.z)) / normal.y
  }

  /// `valid_vertex_position`: the position lies over the map's grid, half a cell around its box.
  pub fn is_valid_position(&self, position: &Vector3d<f32>) -> bool {
    let (min, max) = (&self.header.bounds.min, &self.header.bounds.max);
    let half: f32 = self.header.size * 0.5;

    if position.x < min.x - half || position.x > max.x + half || position.z < min.z - half || position.z > max.z + half
    {
      return false;
    }

    let (x, z) = self.get_cell(position);

    z < self.row_length && x < self.column_length
  }

  /// `CLevelGraph::vertex_id(position)`: among the nodes of the position's cell, the nearest at or under it, else the
  /// nearest over it; `None` where the cell holds no node. The caller checks the position is over the grid.
  pub fn find_node(&self, position: &Vector3d<f32>) -> Option<u32> {
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

  /// A node's cell along x and z, `unpack_xz` into whole cells.
  pub fn get_node_cell(&self, node: u32) -> (i64, i64) {
    let packed: i64 = i64::from(self.nodes[node as usize].packed_xz);

    (packed / self.row_length, packed % self.row_length)
  }

  /// A node's position, `vertex_position`: its cell centre at its packed height.
  pub fn get_node_position(&self, node: u32) -> Vector3d<f32> {
    self.header.get_node_position(&self.nodes[node as usize])
  }

  /// Where a node's cell centre stands across the ground, `unpack_xz`.
  pub fn get_node_xz(&self, node: u32) -> (f32, f32) {
    let position: Vector3d<f32> = self.header.get_node_position(&self.nodes[node as usize]);

    (position.x, position.z)
  }

  /// `CLevelGraph::check_position_in_direction`: the node a straight walk from `start`, standing at `from`, reaches at
  /// `to`, across the ground. Each step takes the first link whose cell the line from `from` toward `to` crosses
  /// (`Fbox2::pick_exact`) and whose cell's middle comes no farther from `to`, never straight back; `None` where no
  /// link does, or `to` lies off the grid. A walk going round cells equally far from `to`, where the engine would never
  /// return, ends unreached once it has taken as many steps as there are nodes.
  pub fn find_straight_walk(&self, start: u32, from: (f32, f32), to: (f32, f32)) -> Option<u32> {
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

        if !AiCellPick::is_crossing(min, max, from, direction) {
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

  /// The packed cell a point across the ground falls in.
  fn get_packed_xz(&self, x: f32, z: f32) -> u32 {
    let (cell_x, cell_z) = self.get_cell(&Vector3d::new(x, 0.0, z));

    (cell_x * self.row_length + cell_z) as u32
  }

  /// The cell a position falls in, `iFloor((v - min) / size + 0.5)` along x and z.
  fn get_cell(&self, position: &Vector3d<f32>) -> (i64, i64) {
    let min = &self.header.bounds.min;

    (
      ((position.x - min.x) / self.header.size + 0.5).floor() as i64,
      ((position.z - min.z) / self.header.size + 0.5).floor() as i64,
    )
  }
}
