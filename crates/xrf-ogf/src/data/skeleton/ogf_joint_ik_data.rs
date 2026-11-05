use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_skeleton::{SkeletonJointKind, SkeletonJointLimit};

/// A bone's joint as an OGF stores it, `SJointIKData::Export`: each limit negated and its two ends swapped for the
/// physics engine's sense of rotation, then the joint's springs, flags, break limits and, from version 1, its
/// friction.
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
  /// Reads the joint of a record stored at `version`: friction follows when its low half is above zero, since the
  /// engine reads the version as a `u32` and narrows it to a `u16` before asking (`SkeletonCustom.cpp`, `Load`).
  pub fn read_versioned<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    version: u32,
  ) -> XrfResult<Self> {
    let is_friction_stored: bool = version & 0xffff > 0;

    Ok(Self {
      kind: SkeletonJointKind::from(reader.read_u32::<T>()?),
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
      friction: if is_friction_stored {
        Some(reader.read_f32::<T>()?)
      } else {
        None
      },
    })
  }

  pub fn write_versioned<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(u32::from(self.kind))?;

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
