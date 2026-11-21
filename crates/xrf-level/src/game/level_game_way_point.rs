use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr()?,
      flags: reader.read_u32()?,
      name: reader.read_w1251_string()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr(&self.position)?;
    writer.write_u32(self.flags)?;
    writer.write_w1251_string(&self.name)?;

    Ok(())
  }
}
