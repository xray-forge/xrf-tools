use byteorder::ByteOrder;
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
#[cfg(test)]
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::XrfResult;

/// A server class payload whose update packet is not empty, read and written across both of an object's packets.
///
/// Each class takes its spawn fields from the spawn packet and its update fields from the update packet, after its
/// base's in each, as its `STATE_Read` and `UPDATE_Read` do. A class whose update packet is empty, and every base it
/// inherits, reads the spawn packet alone through `ChunkFieldReadWrite`. Either packet is bytes or fields, as the
/// engine's `NET_Packet` is with or without an ini stream.
pub(crate) trait AlifePacketsReadWrite: Sized {
  /// Read the payload's spawn fields from `spawn` and its update fields from `update`.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(spawn: &mut S, update: &mut U) -> XrfResult<Self>;

  /// Write the payload's spawn fields into `spawn` and its update fields into `update`.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(&self, spawn: &mut S, update: &mut U) -> XrfResult;

  /// Write the payload and read it back, refusing a byte of either packet left unread, with the sizes of the spawn
  /// and update fields written.
  #[cfg(test)]
  fn read_back<T: ByteOrder>(&self) -> XrfResult<(usize, usize, Self)> {
    let mut spawn: ChunkWriter = ChunkWriter::new();
    let mut update: ChunkWriter = ChunkWriter::new();

    self.write::<T, _, _>(&mut spawn, &mut update)?;

    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer)?;
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update.buffer)?;
    let read: Self = Self::read::<T, _, _>(&mut spawn_reader, &mut update_reader)?;

    spawn_reader.assert_read("Expect the spawn fields to be read whole")?;
    update_reader.assert_read("Expect the update fields to be read whole")?;

    Ok((spawn.buffer.len(), update.buffer.len(), read))
  }
}
