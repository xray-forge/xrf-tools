use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfBone {
  pub name: String,
  pub parent: String,
  pub rotation: (Vector3d, Vector3d, Vector3d),
  pub translate: Vector3d,
  pub half_size: Vector3d,
}

impl OgfBone {
  /// Two terminated names, the rotation triple, the translation, and the half size.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 1 + 36 + 12 + 12;
}

impl ChunkReadWrite for OgfBone {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      name: reader.read_w1251_string()?,
      parent: reader.read_w1251_string()?,
      rotation: (
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ),
      translate: reader.read_xr::<T, _>()?,
      half_size: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_w1251_string(&self.parent)?;

    writer.write_xr::<T, _>(&self.rotation.0)?;
    writer.write_xr::<T, _>(&self.rotation.1)?;
    writer.write_xr::<T, _>(&self.rotation.2)?;

    writer.write_xr::<T, _>(&self.translate)?;
    writer.write_xr::<T, _>(&self.half_size)?;

    Ok(())
  }
}
