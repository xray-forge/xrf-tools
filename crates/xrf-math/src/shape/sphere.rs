use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// A sphere, the engine's `Fsphere`: its centre, `P`, and radius, `R`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sphere {
  pub center: Vector3d,
  pub radius: f32,
}

impl Sphere {
  /// A centre and a radius, four `f32`s.
  pub const SERIALIZED_SIZE: u64 = 16;
}

impl ChunkReadWrite for Sphere {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      center: reader.read_xr::<T, _>()?,
      radius: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.center.write::<T>(writer)?;
    writer.write_f32::<T>(self.radius)?;

    Ok(())
  }
}
