use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

/// One progressive mesh level of detail, `FSlideWindow` in the engine (`FMesh.hpp:109`).
#[derive(Clone, Debug, PartialEq, Eq)]
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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      offset: reader.read_u32()?,
      num_tris: reader.read_u16()?,
      num_verts: reader.read_u16()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.offset)?;
    writer.write_u16(self.num_tris)?;
    writer.write_u16(self.num_verts)?;

    Ok(())
  }
}
