use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf::ogf_joint_limit::OgfJointLimit;

/// Inverse kinematics settings of one bone, `SJointIKData` in the engine.
///
/// Versioned: `friction` was added after version 0, so an older record simply ends before it
/// (`SJointIKData::Import` in `Bone.cpp`). The version is stored per bone in the ik data chunk, which is
/// why reading takes it as an argument rather than inferring it.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfJointIkData {
  /// 0 rigid, 1 cloth, 2 joint, 3 wheel, 4 none, 5 slider.
  pub joint_type: u32,
  /// One per axis, XYZ for a joint and Z-wheel / X-steer for a wheel.
  pub limits: [OgfJointLimit; 3],
  pub spring_factor: f32,
  pub damping_factor: f32,
  pub ik_flags: u32,
  pub break_force: f32,
  pub break_torque: f32,
  /// Absent in version 0 records.
  pub friction: Option<f32>,
}

impl OgfJointIkData {
  pub fn read_versioned<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    version: u16,
  ) -> XrfResult<Self> {
    Ok(Self {
      joint_type: reader.read_u32::<T>()?,
      limits: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
      spring_factor: reader.read_f32::<T>()?,
      damping_factor: reader.read_f32::<T>()?,
      ik_flags: reader.read_u32::<T>()?,
      break_force: reader.read_f32::<T>()?,
      break_torque: reader.read_f32::<T>()?,
      friction: if version > 0 {
        Some(reader.read_f32::<T>()?)
      } else {
        None
      },
    })
  }

  pub fn write_versioned<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.joint_type)?;

    for limit in &self.limits {
      limit.write::<T>(writer)?;
    }

    writer.write_f32::<T>(self.spring_factor)?;
    writer.write_f32::<T>(self.damping_factor)?;
    writer.write_u32::<T>(self.ik_flags)?;
    writer.write_f32::<T>(self.break_force)?;
    writer.write_f32::<T>(self.break_torque)?;

    if let Some(friction) = self.friction {
      writer.write_f32::<T>(friction)?;
    }

    Ok(())
  }
}
