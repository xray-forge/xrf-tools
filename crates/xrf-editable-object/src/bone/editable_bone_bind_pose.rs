use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// A bone's rest pose against its parent, `BONE_CHUNK_BIND_POSE`. Kept as stored: a version 1 bone stores its first
/// two rotation components swapped, which `CBone::Load_1` swaps back on load.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneBindPose {
  pub offset: Vector3d,
  pub rotation: Vector3d,
  pub length: f32,
}

impl ChunkReadWrite for EditableBoneBindPose {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      offset: reader.read_xr::<T, _>()?,
      rotation: reader.read_xr::<T, _>()?,
      length: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.offset.write::<T>(writer)?;
    self.rotation.write::<T>(writer)?;
    writer.write_f32::<T>(self.length)?;

    Ok(())
  }
}
