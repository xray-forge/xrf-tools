use std::io::{Read, Write};

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
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
  pub fn decode<T: ByteOrder>(bytes: &[u8; Self::SIZE as usize]) -> Self {
    Self {
      game_vertex_id: T::read_u16(&bytes[..2]),
      distance: T::read_f32(&bytes[2..]),
    }
  }

  /// Encode this cell into its bytes, the layout [`Self::decode`] reads.
  pub fn encode<T: ByteOrder>(&self) -> [u8; Self::SIZE as usize] {
    let mut bytes: [u8; Self::SIZE as usize] = [0; Self::SIZE as usize];

    T::write_u16(&mut bytes[..2], self.game_vertex_id);
    T::write_f32(&mut bytes[2..], self.distance);

    bytes
  }
}

impl ChunkReadWrite for GraphCrossTableCell {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut bytes: [u8; Self::SIZE as usize] = [0; Self::SIZE as usize];

    reader.read_exact(&mut bytes)?;

    Ok(Self::decode::<T>(&bytes))
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&self.encode::<T>())?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use byteorder::BigEndian;
  use xrf_chunk::XRayByteOrder;

  use crate::data::graph::graph_cross_table_cell::GraphCrossTableCell;

  #[test]
  fn encodes_the_vertex_then_the_distance_packed_to_two() {
    let cell: GraphCrossTableCell = GraphCrossTableCell {
      game_vertex_id: 0x1234,
      distance: 1.0,
    };

    assert_eq!(cell.encode::<XRayByteOrder>(), [0x34, 0x12, 0x00, 0x00, 0x80, 0x3F]);
    assert_eq!(cell.encode::<BigEndian>(), [0x12, 0x34, 0x3F, 0x80, 0x00, 0x00]);
    assert_eq!(
      GraphCrossTableCell::decode::<XRayByteOrder>(&[0x34, 0x12, 0x00, 0x00, 0x80, 0x3F]),
      cell
    );
    assert_eq!(
      GraphCrossTableCell::decode::<BigEndian>(&[0x12, 0x34, 0x3F, 0x80, 0x00, 0x00]),
      cell
    );
  }
}
