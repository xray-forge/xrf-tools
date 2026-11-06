use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Obb;

/// One bone of `OGF_S_BONE_NAMES`: its name, its parent's, and the oriented box the engine reads with them.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfBone {
  pub name: String,
  pub parent: String,
  pub obb: Obb,
}

impl OgfBone {
  /// Two terminated names and the box.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 1 + Obb::SERIALIZED_SIZE;
}

impl ChunkReadWrite for OgfBone {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      name: reader.read_w1251_string()?,
      parent: reader.read_w1251_string()?,
      obb: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_w1251_string(&self.parent)?;
    self.obb.write::<T>(writer)?;

    Ok(())
  }
}
