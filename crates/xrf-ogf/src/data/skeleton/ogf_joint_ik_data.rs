use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_skeleton::{SkeletonJointKind, SkeletonJointLimit};

/// A bone's joint as an OGF stores it, `SJointIKData::Export`: each limit negated and its ends swapped for the
/// physics engine's sense of rotation, and from version 1 a friction.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfJointIkData {
  pub kind: SkeletonJointKind,
  /// One per axis, XYZ for a joint and Z-wheel, X-steer for a wheel.
  pub limits: [SkeletonJointLimit; 3],
  pub spring_factor: f32,
  pub damping_factor: f32,
  pub ik_flags: u32,
  pub break_force: f32,
  pub break_torque: f32,
  /// Absent in version 0 records.
  pub friction: Option<f32>,
}

impl OgfJointIkData {
  /// Reads the joint of a record stored at `version`. Friction follows when the version's low half is above zero, as
  /// the engine narrows it to a `u16` before asking.
  pub fn read_versioned<D: ChunkDataSource>(reader: &mut ChunkReader<D>, version: u32) -> XrfResult<Self> {
    let is_friction_stored: bool = version & 0xffff > 0;

    Ok(Self {
      kind: SkeletonJointKind::from(reader.read_u32()?),
      limits: [reader.read_xr()?, reader.read_xr()?, reader.read_xr()?],
      spring_factor: reader.read_f32()?,
      damping_factor: reader.read_f32()?,
      ik_flags: reader.read_u32()?,
      break_force: reader.read_f32()?,
      break_torque: reader.read_f32()?,
      friction: if is_friction_stored {
        Some(reader.read_f32()?)
      } else {
        None
      },
    })
  }

  pub fn write_versioned(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(u32::from(self.kind))?;

    for limit in &self.limits {
      limit.write(writer)?;
    }

    writer.write_f32(self.spring_factor)?;
    writer.write_f32(self.damping_factor)?;
    writer.write_u32(self.ik_flags)?;
    writer.write_f32(self.break_force)?;
    writer.write_f32(self.break_torque)?;

    if let Some(friction) = self.friction {
      writer.write_f32(friction)?;
    }

    Ok(())
  }
}
