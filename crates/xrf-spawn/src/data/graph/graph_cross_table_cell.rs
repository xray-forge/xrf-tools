use std::io::{Read, Write};

use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayBytes};
use xrf_error::XrfResult;

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
  pub const SIZE: u64 = 2 + 4;

  /// Decodes a cell from its bytes, the one statement of its layout.
  pub fn decode(bytes: &[u8; Self::SIZE as usize]) -> Self {
    Self {
      game_vertex_id: XRayBytes::read_u16(&bytes[..2]),
      distance: XRayBytes::read_f32(&bytes[2..]),
    }
  }

  /// Encode this cell into its bytes, the layout [`Self::decode`] reads.
  pub fn encode(&self) -> [u8; Self::SIZE as usize] {
    let mut bytes: [u8; Self::SIZE as usize] = [0; Self::SIZE as usize];

    XRayBytes::write_u16(&mut bytes[..2], self.game_vertex_id);
    XRayBytes::write_f32(&mut bytes[2..], self.distance);

    bytes
  }
}

impl ChunkReadWrite for GraphCrossTableCell {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut bytes: [u8; Self::SIZE as usize] = [0; Self::SIZE as usize];

    reader.read_exact(&mut bytes)?;

    Ok(Self::decode(&bytes))
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&self.encode())?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
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
