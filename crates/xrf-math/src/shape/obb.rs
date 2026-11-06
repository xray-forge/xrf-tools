use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// An oriented box, the engine's `Fobb` (`xrCore/_obb.h`): a rotation, the centre it is placed at, and half its size
/// along each of its own axes, in the order the engine stores them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Obb {
  /// `m_rotate`, an `Fmatrix33`: its rows are the box's axes.
  pub rotation: [Vector3d; 3],
  /// `m_translate`.
  pub center: Vector3d,
  /// `m_halfsize`.
  pub half_extents: Vector3d,
}

impl Obb {
  /// A 3x3 rotation and two vectors, fifteen `f32`s.
  pub const SERIALIZED_SIZE: u64 = 60;
}

impl ChunkReadWrite for Obb {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      rotation: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
      center: reader.read_xr::<T, _>()?,
      half_extents: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for row in &self.rotation {
      row.write::<T>(writer)?;
    }

    self.center.write::<T>(writer)?;
    self.half_extents.write::<T>(writer)?;

    Ok(())
  }
}
