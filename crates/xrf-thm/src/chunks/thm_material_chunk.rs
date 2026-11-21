use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

use crate::texture::thm_material::ThmMaterial;

/// Shading declaration of a texture, `THM_CHUNK_MATERIAL` in the engine (`ETextureParams.cpp`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmMaterialChunk {
  pub material: ThmMaterial,
  pub weight: f32,
}

impl ThmMaterialChunk {
  pub const CHUNK_ID: u32 = 0x0816;
}

impl Default for ThmMaterialChunk {
  /// `STextureParams::STextureParams` sets the material and zeroes the weight (`ETextureParams.h`).
  fn default() -> Self {
    Self {
      material: ThmMaterial::default(),
      weight: 0.0,
    }
  }
}

impl ChunkReadWrite for ThmMaterialChunk {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunk: Self = Self {
      material: ThmMaterial::from(reader.read_u32()?),
      weight: reader.read_f32()?,
    };

    // Bytes past the fields are the descriptor's to keep, as `ThmFile::trailing`.

    Ok(chunk)
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.material.into())?;
    writer.write_f32(self.weight)?;

    Ok(())
  }
}
