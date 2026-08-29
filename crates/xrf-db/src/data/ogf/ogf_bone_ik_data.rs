use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;
use crate::data::ogf::ogf_bone_shape::OgfBoneShape;
use crate::data::ogf::ogf_joint_ik_data::OgfJointIkData;

/// Physics and bind pose data of one bone, one record of `OGF_S_IKDATA`.
///
/// The bind transform is stored as an euler triple and a translation rather than a matrix; the engine
/// composes it with `setXYZi` then `translate_over` (`SkeletonCustom.cpp`). Kept in the stored form here
/// so writing it back is exact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfBoneIkData {
  /// Per bone, and it selects whether the joint record carries a friction value.
  pub version: u16,
  pub game_material: String,
  pub shape: OgfBoneShape,
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
  pub const MIN_SERIALIZED_SIZE: u64 = 4 + 1 + OgfBoneShape::SIZE as u64;
}

impl ChunkReadWrite for OgfBoneIkData {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    // Stored as u32 even though only the low half is meaningful.
    let version: u16 = reader.read_u32::<T>()? as u16;

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
    writer.write_u32::<T>(self.version as u32)?;

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
