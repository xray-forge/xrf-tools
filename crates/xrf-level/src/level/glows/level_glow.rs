use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One glow of a compiled level: a light flare billboard the static-lighting renderer draws where a light sits.
///
/// R1 alone reads it (`xray-monolith/src/Layers/xrRenderPC_R1/GlowManager.cpp`, `CGlowManager::Load`); R2 and later
/// keep `CGlow` as an empty stub.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGlow {
  pub position: Vector3d<f32>,
  /// The radius it is culled and drawn at.
  pub radius: f32,
  /// The surface it draws with, by its row in the level's shader table: an `effects\glow` entry.
  pub shader: u16,
}

impl LevelGlow {
  /// Bytes one record occupies: a position, a radius and a shader index.
  pub const SERIALIZED_SIZE: u64 = 12 + 4 + 2;
}

impl ChunkReadWrite for LevelGlow {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      radius: reader.read_f32::<T>()?,
      shader: reader.read_u16::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_f32::<T>(self.radius)?;
    writer.write_u16::<T>(self.shader)?;

    Ok(())
  }
}
