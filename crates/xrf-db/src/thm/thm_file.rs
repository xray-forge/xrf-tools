use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, find_optional_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::thm::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::thm::chunks::thm_detail_chunk::ThmDetailChunk;
use crate::thm::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
use crate::thm::chunks::thm_texture_type_chunk::ThmTextureTypeChunk;
use crate::thm::thm_detail_usage::ThmDetailUsage;

/// Texture descriptor file, `STextureParams` in the engine (`ETextureParams.cpp`).
///
/// Reads the chunks with runtime consequences and nothing else. `CTextureDescrMngr::LoadTHM`
/// (`TextureDescrManager.cpp`) acts on the texture type, which gates the whole file, the bump
/// declaration, and the detail association with the two flags that switch it on. Everything else
/// is authoring metadata for the converter, so it stays unparsed and any edit patches raw chunks
/// rather than re-serializing, see [`crate::ThmBumpProcessor`].
#[derive(Debug, Serialize, Deserialize)]
pub struct ThmFile {
  pub texture_param: Option<ThmTextureParamChunk>,
  pub texture_type: Option<ThmTextureTypeChunk>,
  pub detail: Option<ThmDetailChunk>,
  pub bump: Option<ThmBumpChunk>,
}

impl ThmFile {
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "THM file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads a descriptor from a chunk reader over any data source.
  ///
  /// The route an archived descriptor takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      texture_param: Self::read_optional::<T, D, ThmTextureParamChunk>(&chunks, ThmTextureParamChunk::CHUNK_ID)?,
      texture_type: Self::read_optional::<T, D, ThmTextureTypeChunk>(&chunks, ThmTextureTypeChunk::CHUNK_ID)?,
      detail: Self::read_optional::<T, D, ThmDetailChunk>(&chunks, ThmDetailChunk::CHUNK_ID)?,
      bump: Self::read_optional::<T, D, ThmBumpChunk>(&chunks, ThmBumpChunk::CHUNK_ID)?,
    })
  }

  fn read_optional<T: ByteOrder, D: ChunkDataSource, C: ChunkReadWrite>(
    chunks: &[ChunkReader<D>],
    id: u32,
  ) -> XrfResult<Option<C>> {
    match find_optional_chunk_by_id(chunks, id) {
      Some(mut chunk) => Ok(Some(chunk.read_xr::<T, C>()?)),
      None => Ok(None),
    }
  }

  /// The texture type the engine sees, which for a file without the chunk is the zeroed default.
  pub fn texture_type(&self) -> u32 {
    self
      .texture_type
      .as_ref()
      .map_or(ThmTextureTypeChunk::IMAGE, |chunk| chunk.texture_type)
  }

  /// Whether `LoadTHM` reads this descriptor's bump and detail at all.
  pub fn is_described_by_engine(&self) -> bool {
    ThmTextureTypeChunk::is_described_by_engine(self.texture_type())
  }

  /// Bump texture this descriptor asks the engine to resolve, if any.
  ///
  /// Answers from the bump chunk alone; the type gate is the caller's to apply through
  /// [`Self::is_described_by_engine`], because a reader reporting a descriptor wants to say both.
  pub fn used_bump_name(&self) -> Option<&str> {
    self
      .bump
      .as_ref()
      .filter(|bump| bump.is_used())
      .map(|bump| bump.name.as_str())
  }

  /// How the detail chunk is applied, or `None` when the engine would not associate it.
  ///
  /// A detail name without either flag is dead data (`TextureDescrManager.cpp:163` requires
  /// `flags.is_any(flDiffuseDetail | flBumpDetail)`), and so is a flag without a name.
  pub fn used_detail_usage(&self) -> Option<ThmDetailUsage> {
    self.detail.as_ref().filter(|detail| !detail.name.is_empty())?;

    let param: &ThmTextureParamChunk = self.texture_param.as_ref()?;

    match (param.is_diffuse_detail(), param.is_bump_detail()) {
      (true, true) => Some(ThmDetailUsage::DiffuseAndBump),
      (true, false) => Some(ThmDetailUsage::Diffuse),
      (false, true) => Some(ThmDetailUsage::Bump),
      (false, false) => None,
    }
  }
}
