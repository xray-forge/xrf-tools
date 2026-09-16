use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One level of detail of a progressive mesh, `FSlideWindow` (`xrCore/FMesh.hpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomSlideWindow {
  /// Where in the index buffer this level of detail starts.
  pub offset: u32,
  pub triangles: u16,
  pub vertices: u16,
}

impl LevelGeomSlideWindow {
  /// Bytes one record occupies.
  pub const SERIALIZED_SIZE: u64 = 4 + 2 + 2;
}

impl ChunkReadWrite for LevelGeomSlideWindow {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      offset: reader.read_u32::<T>()?,
      triangles: reader.read_u16::<T>()?,
      vertices: reader.read_u16::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.offset)?;
    writer.write_u16::<T>(self.triangles)?;
    writer.write_u16::<T>(self.vertices)?;

    Ok(())
  }
}
