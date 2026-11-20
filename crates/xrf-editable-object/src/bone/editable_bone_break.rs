use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// What breaks a bone's joint, `BONE_CHUNK_IK_JOINT_BREAK`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneBreak {
  pub force: f32,
  pub torque: f32,
}

impl ChunkReadWrite for EditableBoneBreak {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      force: reader.read_f32::<T>()?,
      torque: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.force)?;
    writer.write_f32::<T>(self.torque)?;

    Ok(())
  }
}
