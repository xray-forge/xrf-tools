use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

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
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      color: reader.read_u32::<T>()?,
      u: reader.read_f32::<T>()?,
      v: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_u32::<T>(self.color)?;
    writer.write_f32::<T>(self.u)?;
    writer.write_f32::<T>(self.v)?;

    Ok(())
  }
}
