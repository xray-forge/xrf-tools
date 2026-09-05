use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::thm::thm_material::ThmMaterial;

/// Shading declaration of a texture, `THM_CHUNK_MATERIAL` in the engine (`ETextureParams.cpp`).
///
/// [`Self::material`] names two lighting models and [`Self::weight`] says where between them the surface sits. The
/// pair reaches the renderer through the descriptor manager rather than through the DDS, so unlike the rest of the
/// authoring data it is a property of the texture the engine actually reads.
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
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunk: Self = Self {
      material: ThmMaterial::from(reader.read_u32::<T>()?),
      weight: reader.read_f32::<T>()?,
    };

    reader.assert_read("Expect all data to be read from thm material chunk")?;

    Ok(chunk)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.material.into())?;
    writer.write_f32::<T>(self.weight)?;

    Ok(())
  }
}
