use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One entry a face corner takes from a vertex map, `st_VMapPt`: which map, and which entry in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditableVMapPoint {
  /// Index into the mesh's vertex maps.
  pub vmap: u32,
  /// Index into that map's entries.
  pub index: u32,
}

impl EditableVMapPoint {
  /// Two `int`s.
  pub const SIZE: u64 = 2 * 4;
}

impl ChunkReadWrite for EditableVMapPoint {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vmap: reader.read_u32::<T>()?,
      index: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.vmap)?;
    writer.write_u32::<T>(self.index)?;

    Ok(())
  }
}
