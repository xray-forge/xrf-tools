use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// A sphere, the engine's `Fsphere`: its centre, `P`, and radius, `R`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sphere {
  pub center: Vector3d,
  pub radius: f32,
}

impl Sphere {
  /// A centre and a radius, four `f32`s.
  pub const SERIALIZED_SIZE: u64 = 16;
}

/// The centre, then the radius: as bytes, the raw run the engine writes an `Fsphere` as; as fields, the vector and
/// float `CSE_Shape::cform_write` gives an ini stream.
impl ChunkFieldReadWrite for Sphere {
  #[inline]
  fn read_fields<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self {
      center: reader.read_xr()?,
      radius: reader.read_f32()?,
    })
  }

  #[inline]
  fn write_fields<W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_xr(&self.center)?;
    writer.write_f32(self.radius)
  }
}
