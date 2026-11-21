use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// A cylinder, the engine's `Fcylinder`: its centre, the direction of its axis, its full height along that axis and
/// its radius.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cylinder {
  pub center: Vector3d,
  pub direction: Vector3d,
  pub height: f32,
  pub radius: f32,
}

impl Cylinder {
  /// Two vectors and two `f32`s.
  pub const SERIALIZED_SIZE: u64 = 32;
}

/// `Fcylinder`: centre, axis, height, radius.
impl ChunkFieldReadWrite for Cylinder {
  #[inline]
  fn read_fields<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self {
      center: reader.read_xr()?,
      direction: reader.read_xr()?,
      height: reader.read_f32()?,
      radius: reader.read_f32()?,
    })
  }

  #[inline]
  fn write_fields<W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_xr(&self.center)?;
    writer.write_xr(&self.direction)?;
    writer.write_f32(self.height)?;
    writer.write_f32(self.radius)
  }
}
