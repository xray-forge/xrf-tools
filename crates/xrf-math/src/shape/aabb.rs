use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// An axis-aligned box, the engine's `Fbox`: its lowest and highest corner.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aabb {
  pub min: Vector3d,
  pub max: Vector3d,
}

impl Aabb {
  /// Two corners of three `f32`s.
  pub const SERIALIZED_SIZE: u64 = 24;
}

impl ChunkReadWrite for Aabb {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      min: reader.read_xr::<T, _>()?,
      max: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.min.write::<T>(writer)?;
    self.max.write::<T>(writer)?;

    Ok(())
  }
}
