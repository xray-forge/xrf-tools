use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// What kind of texture a descriptor describes, `THM_CHUNK_TEXTURE_TYPE` in the engine (`ETextureParams.h`).
///
/// The gate on everything else in the file: `CTextureDescrMngr::LoadTHM` (`TextureDescrManager.cpp`) takes the
/// bump, detail and material of a descriptor only when its type is [`Self::IMAGE`], [`Self::NORMAL_MAP`] or
/// [`Self::TERRAIN`]. A cube map or a bump map descriptor contributes nothing, however complete its bump chunk is.
///
/// Absent from a file, the engine's zeroed default is [`Self::IMAGE`], which qualifies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmTextureTypeChunk {
  pub texture_type: u32,
}

impl ThmTextureTypeChunk {
  pub const CHUNK_ID: u32 = 0x0814;

  /// `STextureParams::ETType`, `ETextureParams.h:10`.
  pub const IMAGE: u32 = 0;
  pub const CUBE_MAP: u32 = 1;
  pub const BUMP_MAP: u32 = 2;
  pub const NORMAL_MAP: u32 = 3;
  pub const TERRAIN: u32 = 4;

  /// Whether the engine reads the rest of a descriptor with this type at all.
  pub fn is_described_by_engine(texture_type: u32) -> bool {
    matches!(texture_type, Self::IMAGE | Self::NORMAL_MAP | Self::TERRAIN)
  }

  /// Engine token for a type, `ttype_token` in `ETextureParams.cpp:27`, or the number for one it has no name for.
  pub fn label(texture_type: u32) -> String {
    match texture_type {
      Self::IMAGE => String::from("2D Texture"),
      Self::CUBE_MAP => String::from("Cube Map"),
      Self::BUMP_MAP => String::from("Bump Map"),
      Self::NORMAL_MAP => String::from("Normal Map"),
      Self::TERRAIN => String::from("Terrain"),
      other => other.to_string(),
    }
  }
}

impl ChunkReadWrite for ThmTextureTypeChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunk: Self = Self {
      texture_type: reader.read_u32::<T>()?,
    };

    reader.assert_read("Expect all data to be read from thm texture type chunk")?;

    Ok(chunk)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.texture_type)?;

    Ok(())
  }
}
