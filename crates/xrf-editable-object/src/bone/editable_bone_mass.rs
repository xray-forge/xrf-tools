use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// A bone's mass and where it centres, `BONE_CHUNK_MASS`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneMass {
  pub mass: f32,
  pub center: Vector3d,
}

impl ChunkReadWrite for EditableBoneMass {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      mass: reader.read_f32::<T>()?,
      center: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.mass)?;
    self.center.write::<T>(writer)?;

    Ok(())
  }
}
