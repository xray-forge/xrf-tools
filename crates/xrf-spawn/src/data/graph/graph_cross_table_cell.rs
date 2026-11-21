use xrf_chunk::{ChunkRecord, XRayBytes};

/// One AI node's place in the game graph, `CGameLevelCrossTable::CCell`: its game vertex and its distance from it, in
/// metres along links. A node no graph point reaches keeps the level's first vertex and a distance whose bits are
/// `u32::MAX`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphCrossTableCell {
  /// The game vertex, numbered across the whole game graph.
  pub game_vertex_id: u16,
  pub distance: f32,
}

impl GraphCrossTableCell {
  /// The bytes of one cell, packed to two (`#pragma pack(push,2)`).
  pub const SIZE: usize = 2 + 4;
}

/// The one statement of a cell's layout: its game vertex, then its distance.
impl ChunkRecord<{ Self::SIZE }> for GraphCrossTableCell {
  #[inline]
  fn decode(bytes: &[u8; Self::SIZE]) -> Self {
    Self {
      game_vertex_id: XRayBytes::read_u16(&bytes[..2]),
      distance: XRayBytes::read_f32(&bytes[2..]),
    }
  }

  #[inline]
  fn encode(&self) -> [u8; Self::SIZE] {
    let mut bytes: [u8; Self::SIZE] = [0; Self::SIZE];

    XRayBytes::write_u16(&mut bytes[..2], self.game_vertex_id);
    XRayBytes::write_f32(&mut bytes[2..], self.distance);

    bytes
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::ChunkRecord;

  use crate::data::graph::graph_cross_table_cell::GraphCrossTableCell;

  #[test]
  fn encodes_the_vertex_then_the_distance_packed_to_two() {
    let cell: GraphCrossTableCell = GraphCrossTableCell {
      game_vertex_id: 0x1234,
      distance: 1.0,
    };

    assert_eq!(cell.encode(), [0x34, 0x12, 0x00, 0x00, 0x80, 0x3F]);
    assert_eq!(GraphCrossTableCell::decode(&[0x34, 0x12, 0x00, 0x00, 0x80, 0x3F]), cell);
  }
}
