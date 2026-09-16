use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// One node of a patrol path as the level editor authored it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGameWayPoint {
  pub position: Vector3d<f32>,
  pub flags: u32,
  pub name: String,
}

impl LevelGameWayPoint {
  /// Bytes a node occupies before its name.
  pub const FIXED_SIZE: u64 = 12 + 4;
}

impl ChunkReadWrite for LevelGameWayPoint {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      flags: reader.read_u32::<T>()?,
      name: reader.read_w1251_string()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_w1251_string(&self.name)?;

    Ok(())
  }
}
