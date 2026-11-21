use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One vertex of a baked wallmark, `FVF::LIT` (`Layers/xrRender/FVF.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWallmarkVertex {
  pub position: Vector3d<f32>,
  /// The baked colour, as the packed word the renderer hands straight to the vertex buffer.
  pub color: u32,
  pub u: f32,
  pub v: f32,
}

impl LevelWallmarkVertex {
  /// Bytes a vertex occupies: a position, a packed colour and a texture coordinate.
  pub const SERIALIZED_SIZE: u64 = 12 + 4 + 8;
}

impl ChunkReadWrite for LevelWallmarkVertex {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr()?,
      color: reader.read_u32()?,
      u: reader.read_f32()?,
      v: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr(&self.position)?;
    writer.write_u32(self.color)?;
    writer.write_f32(self.u)?;
    writer.write_f32(self.v)?;

    Ok(())
  }
}
