use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// Oriented bounding box, `Fobb` in the engine.
///
/// Distinct from [`crate::OgfBox`], which is an axis aligned min/max pair. This carries a rotation, so
/// it is 15 floats rather than 6.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfObb {
  /// Row major 3x3 rotation.
  pub rotate: [Vector3d; 3],
  pub translate: Vector3d,
  pub half_size: Vector3d,
}

impl ChunkReadWrite for OgfObb {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      rotate: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
      translate: reader.read_xr::<T, _>()?,
      half_size: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for row in &self.rotate {
      row.write::<T>(writer)?;
    }

    self.translate.write::<T>(writer)?;
    self.half_size.write::<T>(writer)?;

    Ok(())
  }
}
