use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_skeleton::SkeletonBoneShape;

use crate::data::skeleton::ogf_joint_ik_data::OgfJointIkData;

/// Physics and bind pose data of one bone, one record of `OGF_S_IKDATA`.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfBoneIkData {
  /// Per bone, and it selects whether the joint record carries a friction value. Stored as a `u32` whose low half the
  /// engine reads; kept whole, so a record writes back as it was.
  pub version: u32,
  pub game_material: String,
  pub shape: SkeletonBoneShape,
  pub joint: OgfJointIkData,
  /// Bind rotation as euler angles, applied with `setXYZi`.
  pub bind_rotation: Vector3d,
  pub bind_position: Vector3d,
  pub mass: f32,
  pub center_of_mass: Vector3d,
}

impl OgfBoneIkData {
  /// The version, the shortest material name, and the shape blob. The joint record and the transforms that trail it
  /// only make a record longer, so they stay out of the floor.
  pub const MIN_SERIALIZED_SIZE: u64 = 4 + 1 + SkeletonBoneShape::SERIALIZED_SIZE;
}

impl ChunkReadWrite for OgfBoneIkData {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u32 = reader.read_u32::<T>()?;

    Ok(Self {
      version,
      game_material: reader.read_w1251_string()?,
      shape: reader.read_xr::<T, _>()?,
      joint: OgfJointIkData::read_versioned::<T, _>(reader, version)?,
      bind_rotation: reader.read_xr::<T, _>()?,
      bind_position: reader.read_xr::<T, _>()?,
      mass: reader.read_f32::<T>()?,
      center_of_mass: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_w1251_string(&self.game_material)?;
    self.shape.write::<T>(writer)?;
    self.joint.write_versioned::<T>(writer)?;
    self.bind_rotation.write::<T>(writer)?;
    self.bind_position.write::<T>(writer)?;
    writer.write_f32::<T>(self.mass)?;
    self.center_of_mass.write::<T>(writer)?;

    Ok(())
  }
}
