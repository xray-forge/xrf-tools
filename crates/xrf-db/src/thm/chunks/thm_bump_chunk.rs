use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::thm::thm_bump_mode::ThmBumpMode;

/// Bump declaration of a texture, `THM_CHUNK_BUMP` in the engine (`ETextureParams.h:190`).
///
/// The engine reads this at load time in `CTextureDescrMngr::LoadTHM` and takes [`Self::name`]
/// verbatim as the bump texture path. There is no `_bump` naming convention behind it: a texture
/// renamed on import keeps pointing wherever its thm says, so the bump fails to resolve.
///
/// A name that resolves to nothing does not disable bump mapping. `bump_exist` only tests that the
/// name is non-empty, so the renderer still picks the `_bump` shader variant and the loader
/// substitutes `ed\ed_dummy_bump`, logging `! Fallback to default bump map` once per load. The
/// surface ends up flat while paying for the bump path.
///
/// [`Self::virtual_height`] is read by the generator that builds the pair and by nothing at runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmBumpChunk {
  pub virtual_height: f32,
  pub mode: ThmBumpMode,
  /// Bump texture path without extension, engine-style with backslashes. Empty when unused.
  pub name: String,
}

impl ThmBumpChunk {
  pub const CHUNK_ID: u32 = 0x0817;

  /// The height `STextureParams::STextureParams` starts a new descriptor with (`ETextureParams.h`).
  pub const DEFAULT_VIRTUAL_HEIGHT: f32 = 0.05;

  /// Whether the engine will try to resolve [`Self::name`] as a bump texture.
  pub fn is_used(&self) -> bool {
    self.mode.is_used() && !self.name.is_empty()
  }
}

impl Default for ThmBumpChunk {
  fn default() -> Self {
    Self {
      virtual_height: Self::DEFAULT_VIRTUAL_HEIGHT,
      mode: ThmBumpMode::default(),
      name: String::new(),
    }
  }
}

impl ChunkReadWrite for ThmBumpChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let bump: Self = Self {
      virtual_height: reader.read_f32::<T>()?,
      mode: ThmBumpMode::from(reader.read_u32::<T>()?),
      name: reader.read_w1251_string()?,
    };

    reader.assert_read("Expect all data to be read from thm bump chunk")?;

    Ok(bump)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.virtual_height)?;
    writer.write_u32::<T>(self.mode.into())?;
    writer.write_w1251_string(&self.name)?;

    Ok(())
  }
}
