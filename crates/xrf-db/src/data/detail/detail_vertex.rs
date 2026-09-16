use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// One vertex of a detail model, `CDetail::fvfVertexIn` (`Layers/xrRender/IRenderDetailModel.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailVertex {
  pub position: Vector3d<f32>,
  pub u: f32,
  pub v: f32,
}

impl DetailVertex {
  /// Bytes a vertex occupies: a position and a texture coordinate.
  pub const SERIALIZED_SIZE: u64 = 3 * 4 + 2 * 4;
}

impl ChunkReadWrite for DetailVertex {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      u: reader.read_f32::<T>()?,
      v: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_f32::<T>(self.u)?;
    writer.write_f32::<T>(self.v)?;

    Ok(())
  }
}
