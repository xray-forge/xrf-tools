use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelAiHeader, LevelAiNode, LevelAiNodes};
use xrf_math::Vector3d;

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
    let (x, z) = self.get_cell(position);
    let packed: u32 = (x * self.row_length + z) as u32;
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

  /// The cell a position falls in, `iFloor((v - min) / size + 0.5)` along x and z.
  fn get_cell(&self, position: &Vector3d<f32>) -> (i64, i64) {
    let min = &self.header.bounds.min;

    (
      ((position.x - min.x) / self.header.size + 0.5).floor() as i64,
      ((position.z - min.z) / self.header.size + 0.5).floor() as i64,
    )
  }
}
