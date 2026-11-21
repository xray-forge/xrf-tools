use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_skeleton::SkeletonBoneShape;

use crate::data::skeleton::ogf_joint_ik_data::OgfJointIkData;

/// Physics and bind pose data of one bone, one record of `OGF_S_IKDATA`.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfBoneIkData {
  /// Per bone; selects whether the joint record carries a friction value. The engine reads its low half; kept whole so
  /// a record writes back as it was.
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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u32 = reader.read_u32()?;

    Ok(Self {
      version,
      game_material: reader.read_w1251_string()?,
      shape: reader.read_xr()?,
      joint: OgfJointIkData::read_versioned(reader, version)?,
      bind_rotation: reader.read_xr()?,
      bind_position: reader.read_xr()?,
      mass: reader.read_f32()?,
      center_of_mass: reader.read_xr()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.version)?;
    writer.write_w1251_string(&self.game_material)?;
    self.shape.write(writer)?;
    self.joint.write_versioned(writer)?;
    self.bind_rotation.write(writer)?;
    self.bind_position.write(writer)?;
    writer.write_f32(self.mass)?;
    self.center_of_mass.write(writer)?;

    Ok(())
  }
}
