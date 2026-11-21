use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

/// What breaks a bone's joint, `BONE_CHUNK_IK_JOINT_BREAK`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneBreak {
  pub force: f32,
  pub torque: f32,
}

impl ChunkReadWrite for EditableBoneBreak {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      force: reader.read_f32()?,
      torque: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32(self.force)?;
    writer.write_f32(self.torque)?;

    Ok(())
  }
}
