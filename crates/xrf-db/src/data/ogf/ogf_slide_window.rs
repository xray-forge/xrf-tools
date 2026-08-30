use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One progressive mesh level of detail, `FSlideWindow` in the engine (`FMesh.hpp:109`).
///
/// A sliding window into the shared index buffer: rendering a given detail level draws `num_tris`
/// triangles starting at `offset`, touching `num_verts` vertices. Level zero is the most detailed,
/// which is why `FSkinned.cpp` uses `sw[0]` when it wants full geometry.
///
/// Exactly 8 bytes, so the count in the chunk header and the payload length must agree.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfSlideWindow {
  /// Start position in the index buffer.
  pub offset: u32,
  pub num_tris: u16,
  pub num_verts: u16,
}

impl OgfSlideWindow {
  pub const MIN_SERIALIZED_SIZE: u64 = 4 + 2 + 2;
}

impl ChunkReadWrite for OgfSlideWindow {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      offset: reader.read_u32::<T>()?,
      num_tris: reader.read_u16::<T>()?,
      num_verts: reader.read_u16::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.offset)?;
    writer.write_u16::<T>(self.num_tris)?;
    writer.write_u16::<T>(self.num_verts)?;

    Ok(())
  }
}
