use byteorder::ByteOrder;
#[cfg(test)]
use xrf_chunk::InMemoryChunkDataSource;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// A server class payload whose update packet is not empty, read and written across both of an object's packets.
///
/// Each class takes its spawn fields from the spawn packet and its update fields from the update packet, after its
/// base's in each, as its `STATE_Read` and `UPDATE_Read` do. A class whose update packet is empty, and every base it
/// inherits, reads the spawn packet alone through `ChunkReadWrite`.
pub trait AlifePacketsReadWrite: Sized {
  /// Read the payload's spawn fields from `spawn` and its update fields from `update`.
  fn read<T: ByteOrder, D: ChunkDataSource>(spawn: &mut ChunkReader<D>, update: &mut ChunkReader<D>)
  -> XrfResult<Self>;

  /// Write the payload's spawn fields into `spawn` and its update fields into `update`.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult;

  /// Write the payload and read it back, refusing a byte of either packet left unread, with the sizes of the spawn
  /// and update fields written.
  #[cfg(test)]
  fn read_back<T: ByteOrder>(&self) -> XrfResult<(usize, usize, Self)> {
    let mut spawn: ChunkWriter = ChunkWriter::new();
    let mut update: ChunkWriter = ChunkWriter::new();

    self.write::<T>(&mut spawn, &mut update)?;

    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer)?;
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update.buffer)?;
    let read: Self = Self::read::<T, _>(&mut spawn_reader, &mut update_reader)?;

    spawn_reader.assert_read("Expect the spawn fields to be read whole")?;
    update_reader.assert_read("Expect the update fields to be read whole")?;

    Ok((spawn.buffer.len(), update.buffer.len(), read))
  }
}
