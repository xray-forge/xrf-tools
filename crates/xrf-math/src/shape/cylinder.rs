use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// A cylinder, the engine's `Fcylinder`: its centre, the direction of its axis, its full height along that axis and
/// its radius.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cylinder {
  pub center: Vector3d,
  pub direction: Vector3d,
  pub height: f32,
  pub radius: f32,
}

impl Cylinder {
  /// Two vectors and two `f32`s.
  pub const SERIALIZED_SIZE: u64 = 32;
}

impl ChunkReadWrite for Cylinder {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      center: reader.read_xr::<T, _>()?,
      direction: reader.read_xr::<T, _>()?,
      height: reader.read_f32::<T>()?,
      radius: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.center.write::<T>(writer)?;
    self.direction.write::<T>(writer)?;
    writer.write_f32::<T>(self.height)?;
    writer.write_f32::<T>(self.radius)?;

    Ok(())
  }
}
