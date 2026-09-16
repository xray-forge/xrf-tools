use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// One compiled light source, `R_Light` (`utils/xrLC_Light/R_light.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelLight {
  /// `D3DLIGHT_POINT` is 1 and `D3DLIGHT_DIRECTIONAL` is 3; `CLight_DB::LoadHemi` takes only the point ones.
  pub kind: u16,
  /// Global illumination level the compiler bounced this light at.
  pub level: u16,
  pub diffuse: Vector3d<f32>,
  pub position: Vector3d<f32>,
  pub direction: Vector3d<f32>,
  pub range: f32,
  /// The range squared, precomputed by the compiler.
  pub range_squared: f32,
  pub falloff: f32,
  pub attenuation_constant: f32,
  pub attenuation_linear: f32,
  pub attenuation_quadratic: f32,
  /// Radiosity energy, which the runtime never reads.
  pub energy: f32,
  /// The triangle the light was emitted from, which only the compiler's own bounce pass uses.
  pub triangle: [Vector3d<f32>; 3],
}

impl LevelLight {
  /// `D3DLIGHT_POINT`, the only kind `CLight_DB::LoadHemi` turns into a runtime light.
  pub const KIND_POINT: u16 = 1;

  /// Bytes one light occupies, which is what tells a light chunk from any other.
  pub const SERIALIZED_SIZE: u64 = 2 + 2 + 12 + 12 + 12 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 36;

  /// Whether the runtime would make a light of this, which only a point light becomes.
  pub const fn is_point(&self) -> bool {
    self.kind == Self::KIND_POINT
  }
}

impl ChunkReadWrite for LevelLight {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      kind: reader.read_u16::<T>()?,
      level: reader.read_u16::<T>()?,
      diffuse: reader.read_xr::<T, _>()?,
      position: reader.read_xr::<T, _>()?,
      direction: reader.read_xr::<T, _>()?,
      range: reader.read_f32::<T>()?,
      range_squared: reader.read_f32::<T>()?,
      falloff: reader.read_f32::<T>()?,
      attenuation_constant: reader.read_f32::<T>()?,
      attenuation_linear: reader.read_f32::<T>()?,
      attenuation_quadratic: reader.read_f32::<T>()?,
      energy: reader.read_f32::<T>()?,
      triangle: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.kind)?;
    writer.write_u16::<T>(self.level)?;
    writer.write_xr::<T, _>(&self.diffuse)?;
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_xr::<T, _>(&self.direction)?;
    writer.write_f32::<T>(self.range)?;
    writer.write_f32::<T>(self.range_squared)?;
    writer.write_f32::<T>(self.falloff)?;
    writer.write_f32::<T>(self.attenuation_constant)?;
    writer.write_f32::<T>(self.attenuation_linear)?;
    writer.write_f32::<T>(self.attenuation_quadratic)?;
    writer.write_f32::<T>(self.energy)?;

    for vertex in &self.triangle {
      writer.write_xr::<T, _>(vertex)?;
    }

    Ok(())
  }
}
