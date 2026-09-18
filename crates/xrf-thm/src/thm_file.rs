use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::{ByteOrder, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{
  ChunkDataSource, ChunkReader, ChunkWriter, read_u8_chunk, read_u16_chunk, read_u32_chunk, read_w1251_string_chunk,
};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, open_export_file};

use crate::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::chunks::thm_detail_chunk::ThmDetailChunk;
use crate::chunks::thm_extra_chunk::ThmExtraChunk;
use crate::chunks::thm_material_chunk::ThmMaterialChunk;
use crate::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
use crate::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
use crate::thm_detail_usage::ThmDetailUsage;
use crate::thm_texture_flag::ThmTextureFlag;
use crate::thm_texture_type::ThmTextureType;

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
    }
  }
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

  /// Reads a descriptor from bytes already in hand.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads a descriptor from a chunk reader over any data source.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut file: Self = Self::default();

    for mut chunk in reader.read_children_including_compressed()? {
      let is_folded: bool = match chunk.id {
        Self::VERSION_CHUNK_ID if file.version.is_none() => {
          file.version = Some(read_u16_chunk::<T, _>(&mut chunk)?);
          true
        }
        id if ThmThumbnailChunk::matches(id) && file.thumbnail.is_none() => {
          file.thumbnail = Some(ThmThumbnailChunk::read(&mut chunk, id)?);
          true
        }
        Self::THUMBNAIL_TYPE_CHUNK_ID if file.thumbnail_type.is_none() => {
          file.thumbnail_type = Some(read_u32_chunk::<T, _>(&mut chunk)?);
          true
        }
        ThmTextureParamChunk::CHUNK_ID if file.texture_param.is_none() => {
          file.texture_param = Some(chunk.read_xr::<T, _>()?);
          true
        }
        Self::TEXTURE_TYPE_CHUNK_ID if file.texture_type.is_none() => {
          file.texture_type = Some(ThmTextureType::from(read_u32_chunk::<T, _>(&mut chunk)?));
          true
        }
        ThmDetailChunk::CHUNK_ID if file.detail.is_none() => {
          file.detail = Some(chunk.read_xr::<T, _>()?);
          true
        }
        ThmMaterialChunk::CHUNK_ID if file.material.is_none() => {
          file.material = Some(chunk.read_xr::<T, _>()?);
          true
        }
        ThmBumpChunk::CHUNK_ID if file.bump.is_none() => {
          file.bump = Some(chunk.read_xr::<T, _>()?);
          true
        }
        Self::EXT_NORMAL_MAP_CHUNK_ID if file.ext_normal_map_name.is_none() => {
          file.ext_normal_map_name = Some(read_w1251_string_chunk(&mut chunk)?);
          true
        }
        Self::FADE_DELAY_CHUNK_ID if file.fade_delay.is_none() => {
          file.fade_delay = Some(read_u8_chunk(&mut chunk)?);
          true
        }
        _ => false,
      };

      if !is_folded {
        file.extra.push(ThmExtraChunk {
          id: chunk.id,
          data: chunk.read_remaining()?,
        });
      }
    }

    Ok(file)
  }
}

impl ThmFile {
  /// Writes the descriptor to a path, creating the directories leading to it.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    if let Some(parent) = path.as_ref().parent() {
      fs::create_dir_all(parent)?;
    }

    self.write_to::<T>(&mut open_export_file(path)?)
  }

  /// The descriptor as the bytes of a file.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    self.write_to::<T>(&mut bytes)?;

    Ok(bytes)
  }

  /// Writes the descriptor into the writer.
  pub fn write_to<T: ByteOrder>(&self, destination: &mut dyn Write) -> XrfResult {
    if let Some(version) = self.version {
      Self::write_chunk::<T>(destination, Self::VERSION_CHUNK_ID, |writer| {
        Ok(writer.write_u16::<T>(version)?)
      })?;
    }

    if let Some(thumbnail) = &self.thumbnail {
      Self::write_chunk::<T>(destination, thumbnail.to_chunk_id(), |writer| thumbnail.write(writer))?;
    }

    if let Some(thumbnail_type) = self.thumbnail_type {
      Self::write_chunk::<T>(destination, Self::THUMBNAIL_TYPE_CHUNK_ID, |writer| {
        Ok(writer.write_u32::<T>(thumbnail_type)?)
      })?;
    }

    if let Some(texture_param) = &self.texture_param {
      Self::write_chunk::<T>(destination, ThmTextureParamChunk::CHUNK_ID, |writer| {
        writer.write_xr::<T, _>(texture_param)
      })?;
    }

    if let Some(texture_type) = self.texture_type {
      Self::write_chunk::<T>(destination, Self::TEXTURE_TYPE_CHUNK_ID, |writer| {
        Ok(writer.write_u32::<T>(texture_type.into())?)
      })?;
    }

    if let Some(detail) = &self.detail {
      Self::write_chunk::<T>(destination, ThmDetailChunk::CHUNK_ID, |writer| {
        writer.write_xr::<T, _>(detail)
      })?;
    }

    if let Some(material) = &self.material {
      Self::write_chunk::<T>(destination, ThmMaterialChunk::CHUNK_ID, |writer| {
        writer.write_xr::<T, _>(material)
      })?;
    }

    if let Some(bump) = &self.bump {
      Self::write_chunk::<T>(destination, ThmBumpChunk::CHUNK_ID, |writer| {
        writer.write_xr::<T, _>(bump)
      })?;
    }

    if let Some(name) = &self.ext_normal_map_name {
      Self::write_chunk::<T>(destination, Self::EXT_NORMAL_MAP_CHUNK_ID, |writer| {
        writer.write_w1251_string(name)?;

        Ok(())
      })?;
    }

    if let Some(fade_delay) = self.fade_delay {
      Self::write_chunk::<T>(destination, Self::FADE_DELAY_CHUNK_ID, |writer| {
        Ok(writer.write_u8(fade_delay)?)
      })?;
    }

    for extra in &self.extra {
      Self::write_chunk::<T>(destination, extra.id, |writer| extra.write(writer))?;
    }

    Ok(())
  }

  /// Frames one payload as a chunk of `id`, so that the order above reads as the order on disk.
  fn write_chunk<T: ByteOrder>(
    destination: &mut dyn Write,
    id: u32,
    fill: impl FnOnce(&mut ChunkWriter) -> XrfResult,
  ) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    fill(&mut writer)?;
    writer.flush_chunk_into::<T>(destination, id)?;

    Ok(())
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
