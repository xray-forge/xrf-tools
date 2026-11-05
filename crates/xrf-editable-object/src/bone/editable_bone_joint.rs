use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ogf::OgfJointLimit;

/// How a bone's joint moves, `BONE_CHUNK_IK_JOINT`: the part of `SJointIKData` the editor stores in that chunk, with
/// its break limits and friction in chunks of their own.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneJoint {
  /// `EJointType`: 0 rigid, 1 cloth, 2 joint, 3 wheel, 4 none, 5 slider.
  pub joint_type: u32,
  /// One per axis, XYZ for a joint and Z-wheel, X-steer for a wheel, as the editor holds them: an OGF stores each
  /// limit negated and swapped for the physics engine.
  pub limits: [OgfJointLimit; 3],
  pub spring_factor: f32,
  pub damping_factor: f32,
}

impl ChunkReadWrite for EditableBoneJoint {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      joint_type: reader.read_u32::<T>()?,
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
    writer.write_u32::<T>(self.joint_type)?;

    for limit in &self.limits {
      limit.write::<T>(writer)?;
    }

    writer.write_f32::<T>(self.spring_factor)?;
    writer.write_f32::<T>(self.damping_factor)?;

    Ok(())
  }
}
