use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      offset: reader.read_u32()?,
      triangles: reader.read_u16()?,
      vertices: reader.read_u16()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.offset)?;
    writer.write_u16(self.triangles)?;
    writer.write_u16(self.vertices)?;

    Ok(())
  }
}
