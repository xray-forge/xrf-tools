use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One AI node's place in the game graph, `CGameLevelCrossTable::CCell`: the game vertex the node belongs to and how far
/// it lies from it, in metres along links. A node no graph point reaches keeps the level's first vertex and the
/// distance of `u32::MAX` cells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphCrossTableCell {
  /// The game vertex, numbered across the whole game graph.
  pub game_vertex_id: u16,
  pub distance: f32,
}

impl GraphCrossTableCell {
  /// The bytes of one cell, packed to two (`#pragma pack(push,2)`).
  pub const SIZE: u64 = 2 + 4;
}

impl ChunkReadWrite for GraphCrossTableCell {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      game_vertex_id: reader.read_u16::<T>()?,
      distance: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.game_vertex_id)?;
    writer.write_f32::<T>(self.distance)?;

    Ok(())
  }
}
