use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_skeleton::{SkeletonJointKind, SkeletonJointLimit};

/// How a bone's joint moves, `BONE_CHUNK_IK_JOINT`: the part of `SJointIKData` the editor stores in that chunk, with
/// its break limits and friction in chunks of their own.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneJoint {
  pub kind: SkeletonJointKind,
  /// One per axis, XYZ for a joint and Z-wheel, X-steer for a wheel, as the editor holds them: an OGF stores each
  /// limit negated and swapped for the physics engine.
  pub limits: [SkeletonJointLimit; 3],
  pub spring_factor: f32,
  pub damping_factor: f32,
}

impl ChunkReadWrite for EditableBoneJoint {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      kind: SkeletonJointKind::from(reader.read_u32::<T>()?),
      limits: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
      spring_factor: reader.read_f32::<T>()?,
      damping_factor: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(u32::from(self.kind))?;

    for limit in &self.limits {
      limit.write::<T>(writer)?;
    }

    writer.write_f32::<T>(self.spring_factor)?;
    writer.write_f32::<T>(self.damping_factor)?;

    Ok(())
  }
}
