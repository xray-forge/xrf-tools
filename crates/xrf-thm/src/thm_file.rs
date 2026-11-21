use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::chunks::thm_chunks::ThmChunks;
use crate::chunks::thm_detail_chunk::ThmDetailChunk;
use crate::chunks::thm_extra_chunk::ThmExtraChunk;
use crate::chunks::thm_material_chunk::ThmMaterialChunk;
use crate::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
use crate::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
use crate::chunks::thm_trailing_bytes::ThmTrailingBytes;
use crate::texture::thm_detail_usage::ThmDetailUsage;
use crate::texture::thm_texture_flag::ThmTextureFlag;
use crate::texture::thm_texture_type::ThmTextureType;

/// Texture descriptor file: `ETextureThumbnail` wrapping `STextureParams` (`EThumbnailTexture.cpp`,
/// `ETextureParams.cpp`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmFile {
  pub version: Option<u16>,
  pub thumbnail: Option<ThmThumbnailChunk>,
  pub thumbnail_type: Option<u32>,
  pub texture_param: Option<ThmTextureParamChunk>,
  pub texture_type: Option<ThmTextureType>,
  pub detail: Option<ThmDetailChunk>,
  pub material: Option<ThmMaterialChunk>,
  pub bump: Option<ThmBumpChunk>,
  /// Normal map replacing the one the generator would derive, read at generation time and never at runtime.
  pub ext_normal_map_name: Option<String>,
  /// Mip level the converter starts fading from, `fade_delay` (`ETextureParams.h:88`).
  pub fade_delay: Option<u8>,
  /// What the reader could not fold into a field; see [`ThmExtraChunk`].
  pub extra: Vec<ThmExtraChunk>,
  /// What folded chunks hold past their fields; see [`ThmTrailingBytes`].
  pub trailing: Vec<ThmTrailingBytes>,
}

impl ThmFile {
  /// `THM_CHUNK_VERSION` (`ETextureParams.h`).
  pub const VERSION_CHUNK_ID: u32 = 0x0810;
  /// `THM_CHUNK_TYPE` (`ETextureParams.h`), the kind of asset the thumbnail describes.
  pub const THUMBNAIL_TYPE_CHUNK_ID: u32 = 0x0813;
  /// `THM_CHUNK_TEXTURE_TYPE` (`ETextureParams.h`).
  pub const TEXTURE_TYPE_CHUNK_ID: u32 = 0x0814;
  /// `THM_CHUNK_EXT_NORMALMAP` (`ETextureParams.h`).
  pub const EXT_NORMAL_MAP_CHUNK_ID: u32 = 0x0818;
  /// `THM_CHUNK_FADE_DELAY` (`ETextureParams.h`).
  pub const FADE_DELAY_CHUNK_ID: u32 = 0x0819;

  /// `THM_TEXTURE_VERSION`, the only version `ETextureThumbnail::Load` accepts (`EThumbnailTexture.cpp`).
  pub const VERSION: u16 = 0x0012;

  /// `ECustomThumbnail::ETTexture`, the only subject these tools describe (`EThumbnail.h`).
  pub const THUMBNAIL_TYPE_TEXTURE: u32 = 1;

  /// A descriptor for a texture that has none, with the values the SDK starts one at.
  pub fn new_texture() -> Self {
    Self {
      version: Some(Self::VERSION),
      thumbnail: None,
      thumbnail_type: Some(Self::THUMBNAIL_TYPE_TEXTURE),
      texture_param: Some(ThmTextureParamChunk::default()),
      texture_type: Some(ThmTextureType::default()),
      detail: Some(ThmDetailChunk::default()),
      material: Some(ThmMaterialChunk::default()),
      bump: Some(ThmBumpChunk::default()),
      ext_normal_map_name: Some(String::new()),
      fade_delay: Some(0),
      extra: Vec::new(),
      trailing: Vec::new(),
    }
  }
}

impl ThmFile {
  pub fn read_from_path<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "THM file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  pub fn read_from_file(file: File) -> XrfResult<Self> {
    Self::read_from_chunk(&mut ChunkReader::from_file(file)?)
  }

  /// Reads a descriptor from bytes already in hand.
  pub fn read_from_bytes(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads a descriptor from a chunk reader over any data source, keeping what a chunk holds past its fields.
  pub fn read_from_chunk<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut file: Self = Self::default();

    for mut chunk in reader.read_children_including_compressed()? {
      let is_folded: bool = match chunk.id {
        Self::VERSION_CHUNK_ID if file.version.is_none() => {
          file.version = Some(chunk.read_u16()?);
          true
        }
        id if ThmThumbnailChunk::matches(id) && file.thumbnail.is_none() => {
          file.thumbnail = Some(ThmThumbnailChunk::read(&mut chunk, id)?);
          true
        }
        Self::THUMBNAIL_TYPE_CHUNK_ID if file.thumbnail_type.is_none() => {
          file.thumbnail_type = Some(chunk.read_u32()?);
          true
        }
        ThmTextureParamChunk::CHUNK_ID if file.texture_param.is_none() => {
          file.texture_param = Some(chunk.read_xr()?);
          true
        }
        Self::TEXTURE_TYPE_CHUNK_ID if file.texture_type.is_none() => {
          file.texture_type = Some(ThmTextureType::from(chunk.read_u32()?));
          true
        }
        ThmDetailChunk::CHUNK_ID if file.detail.is_none() => {
          file.detail = Some(chunk.read_xr()?);
          true
        }
        ThmMaterialChunk::CHUNK_ID if file.material.is_none() => {
          file.material = Some(chunk.read_xr()?);
          true
        }
        ThmBumpChunk::CHUNK_ID if file.bump.is_none() => {
          file.bump = Some(chunk.read_xr()?);
          true
        }
        Self::EXT_NORMAL_MAP_CHUNK_ID if file.ext_normal_map_name.is_none() => {
          file.ext_normal_map_name = Some(chunk.read_w1251_string()?);
          true
        }
        Self::FADE_DELAY_CHUNK_ID if file.fade_delay.is_none() => {
          file.fade_delay = Some(chunk.read_u8()?);
          true
        }
        _ => false,
      };

      ThmChunks::keep_rest(&mut chunk, is_folded, &mut file.extra, &mut file.trailing)?;
    }

    Ok(file)
  }
}

impl ThmFile {
  /// Writes the descriptor to a path, creating the directories leading to it.
  pub fn write_to_path<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    Ok(StagedFile::write_creating_directories(
      path.as_ref(),
      &self.write_to_bytes()?,
    )?)
  }

  /// The descriptor as the bytes of a file.
  pub fn write_to_bytes(&self) -> XrfResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    self.write_to(&mut bytes)?;

    Ok(bytes)
  }

  /// Writes the descriptor into the writer.
  pub fn write_to(&self, destination: &mut dyn Write) -> XrfResult {
    if let Some(version) = self.version {
      self.write_chunk(destination, Self::VERSION_CHUNK_ID, |writer| {
        Ok(writer.write_u16(version)?)
      })?;
    }

    if let Some(thumbnail) = &self.thumbnail {
      self.write_chunk(destination, thumbnail.to_chunk_id(), |writer| thumbnail.write(writer))?;
    }

    if let Some(thumbnail_type) = self.thumbnail_type {
      self.write_chunk(destination, Self::THUMBNAIL_TYPE_CHUNK_ID, |writer| {
        Ok(writer.write_u32(thumbnail_type)?)
      })?;
    }

    if let Some(texture_param) = &self.texture_param {
      self.write_chunk(destination, ThmTextureParamChunk::CHUNK_ID, |writer| {
        writer.write_xr(texture_param)
      })?;
    }

    if let Some(texture_type) = self.texture_type {
      self.write_chunk(destination, Self::TEXTURE_TYPE_CHUNK_ID, |writer| {
        Ok(writer.write_u32(texture_type.into())?)
      })?;
    }

    if let Some(detail) = &self.detail {
      self.write_chunk(destination, ThmDetailChunk::CHUNK_ID, |writer| writer.write_xr(detail))?;
    }

    if let Some(material) = &self.material {
      self.write_chunk(destination, ThmMaterialChunk::CHUNK_ID, |writer| {
        writer.write_xr(material)
      })?;
    }

    if let Some(bump) = &self.bump {
      self.write_chunk(destination, ThmBumpChunk::CHUNK_ID, |writer| writer.write_xr(bump))?;
    }

    if let Some(name) = &self.ext_normal_map_name {
      self.write_chunk(destination, Self::EXT_NORMAL_MAP_CHUNK_ID, |writer| {
        writer.write_w1251_string(name)?;

        Ok(())
      })?;
    }

    if let Some(fade_delay) = self.fade_delay {
      self.write_chunk(destination, Self::FADE_DELAY_CHUNK_ID, |writer| {
        Ok(writer.write_u8(fade_delay)?)
      })?;
    }

    ThmChunks::write_extra(destination, &self.extra)
  }

  /// Frames a folded chunk's fields as a chunk of `id`, the bytes read past them after, so that the order above reads
  /// as the order on disk.
  fn write_chunk(
    &self,
    destination: &mut dyn Write,
    id: u32,
    fill: impl FnOnce(&mut ChunkWriter) -> XrfResult,
  ) -> XrfResult {
    ThmChunks::write(destination, id, &self.trailing, fill)
  }
}

impl ThmFile {
  /// Whether the file declares the version `ETextureThumbnail::Load` accepts.
  pub fn is_supported_version(&self) -> bool {
    self.version.is_none_or(|version| version == Self::VERSION)
  }

  /// Whether the file describes a texture rather than one of the other thumbnail subjects.
  pub fn is_texture_thumbnail(&self) -> bool {
    self
      .thumbnail_type
      .is_none_or(|thumbnail_type| thumbnail_type == Self::THUMBNAIL_TYPE_TEXTURE)
  }

  /// The texture type the engine sees, which for a file without the chunk is the zeroed default.
  pub fn texture_type(&self) -> ThmTextureType {
    self.texture_type.unwrap_or_default()
  }

  /// Gives the descriptor `texture_type` as the SDK's editor does, with the flags, format and mip filter that type
  /// brings; see [`ThmTextureParamChunk::apply_texture_type`].
  pub fn change_texture_type(&mut self, texture_type: ThmTextureType) {
    self.texture_type = Some(texture_type);

    if let Some(param) = self.texture_param.as_mut() {
      param.apply_texture_type(texture_type);
    }
  }

  /// Whether `LoadTHM` reads this descriptor's bump and detail at all.
  pub fn is_described_by_engine(&self) -> bool {
    self.texture_type().is_described_by_engine()
  }

  /// Bump texture this descriptor asks the engine to resolve, if any.
  pub fn used_bump_name(&self) -> Option<&str> {
    self
      .bump
      .as_ref()
      .filter(|bump| bump.is_used())
      .map(|bump| bump.name.as_str())
  }

  /// How the detail chunk is applied, or `None` when the engine would not associate it.
  pub fn used_detail_usage(&self) -> Option<ThmDetailUsage> {
    self.detail.as_ref().filter(|detail| !detail.name.is_empty())?;

    let param: &ThmTextureParamChunk = self.texture_param.as_ref()?;

    match (
      param.flags.has(ThmTextureFlag::DiffuseDetail),
      param.flags.has(ThmTextureFlag::BumpDetail),
    ) {
      (true, true) => Some(ThmDetailUsage::DiffuseAndBump),
      (true, false) => Some(ThmDetailUsage::Diffuse),
      (false, true) => Some(ThmDetailUsage::Bump),
      (false, false) => None,
    }
  }
}
