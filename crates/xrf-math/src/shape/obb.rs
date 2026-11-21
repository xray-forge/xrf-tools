use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// An oriented box, the engine's `Fobb`: a rotation, its centre, and half its size along each of its own axes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Obb {
  /// `m_rotate`, an `Fmatrix33`: its rows are the box's axes.
  pub rotation: [Vector3d; 3],
  /// `m_translate`.
  pub center: Vector3d,
  /// `m_halfsize`.
  pub half_extents: Vector3d,
}

impl Obb {
  /// A 3x3 rotation and two vectors, fifteen `f32`s.
  pub const SERIALIZED_SIZE: u64 = 60;
}

/// `Fobb`: the rotation's rows, the centre, the half sizes, each a vector.
impl ChunkFieldReadWrite for Obb {
  #[inline]
  fn read_fields<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self {
      rotation: [reader.read_xr()?, reader.read_xr()?, reader.read_xr()?],
      center: reader.read_xr()?,
      half_extents: reader.read_xr()?,
    })
  }

  #[inline]
  fn write_fields<W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    for row in &self.rotation {
      writer.write_xr(row)?;
    }

    writer.write_xr(&self.center)?;
    writer.write_xr(&self.half_extents)
  }
}
