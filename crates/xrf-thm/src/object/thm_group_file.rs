use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path, to_format_size};

use crate::chunks::thm_chunks::ThmChunks;
use crate::chunks::thm_extra_chunk::ThmExtraChunk;
use crate::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
use crate::chunks::thm_trailing_bytes::ThmTrailingBytes;
use crate::thm_file::ThmFile;

/// A group's thumbnail, `EGroupThumbnail` (`ECore/Editor/EThumbnailGroup.cpp`): the `.thm` beside a `.group`, its
/// preview picture and the objects it gathers. Only the SDK reads it.
///
/// The SDK marks a group's thumbnail as an object's (`ETObject`), so its own version and objects chunk are what tell
/// the two apart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmGroupFile {
  /// The preview the group library shows, see [`ThmThumbnailChunk::decode`].
  pub thumbnail: ThmThumbnailChunk,
  /// The objects the group gathers, by their library names.
  pub objects: Vec<String>,
  /// What the reader could not fold into a field; see [`ThmExtraChunk`].
  pub extra: Vec<ThmExtraChunk>,
  /// What folded chunks hold past their fields; see [`ThmTrailingBytes`].
  pub trailing: Vec<ThmTrailingBytes>,
}

impl ThmGroupFile {
  /// `THM_GROUP_VERSION`, the only version `EGroupThumbnail::Load` accepts.
  pub const VERSION: u16 = 0x0001;

  /// `ECustomThumbnail::ETObject`, which the SDK stamps on a group's thumbnail too.
  pub const THUMBNAIL_TYPE: u32 = 0;

  /// `THM_CHUNK_GROUPPARAM`: the objects' count, then their names.
  pub const OBJECTS_CHUNK_ID: u32 = 0x0001;

  /// Reads a group thumbnail from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a group thumbnail [`Self::read_from_chunk`] reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Group thumbnail was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)?)
  }

  /// Reads a group thumbnail from bytes already in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a group thumbnail [`Self::read_from_chunk`] reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads a group thumbnail as `EGroupThumbnail::Load` does.
  ///
  /// # Errors
  ///
  /// Returns an error, where the SDK fails or asserts, for a version other than [`Self::VERSION`], a type other than
  /// [`Self::THUMBNAIL_TYPE`], a missing preview or objects chunk, or a chunk too short for its fields.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut thumbnail_type: Option<u32> = None;
    let mut thumbnail: Option<ThmThumbnailChunk> = None;
    let mut objects: Option<Vec<String>> = None;
    let mut extra: Vec<ThmExtraChunk> = Vec::new();
    let mut trailing: Vec<ThmTrailingBytes> = Vec::new();

    for mut chunk in reader.read_children_including_compressed()? {
      let is_folded: bool = match chunk.id {
        ThmFile::VERSION_CHUNK_ID if version.is_none() => {
          version = Some(chunk.read_u16::<T>()?);
          true
        }
        id if ThmThumbnailChunk::matches(id) && thumbnail.is_none() => {
          thumbnail = Some(ThmThumbnailChunk::read(&mut chunk, id)?);
          true
        }
        ThmFile::THUMBNAIL_TYPE_CHUNK_ID if thumbnail_type.is_none() => {
          thumbnail_type = Some(chunk.read_u32::<T>()?);
          true
        }
        Self::OBJECTS_CHUNK_ID if objects.is_none() => {
          let count: u32 = chunk.read_u32::<T>()?;

          objects = Some(
            (0..count)
              .map(|_| chunk.read_w1251_string())
              .collect::<XrfResult<_>>()?,
          );
          true
        }
        _ => false,
      };

      ThmChunks::keep_rest(&mut chunk, is_folded, &mut extra, &mut trailing)?;
    }

    ThmChunks::require_kind("a group", version, Self::VERSION, thumbnail_type, Self::THUMBNAIL_TYPE)?;

    Ok(Self {
      thumbnail: thumbnail.ok_or_else(|| XrfError::new_parsing_error("Group thumbnail carries no preview"))?,
      objects: objects.ok_or_else(|| XrfError::new_parsing_error("Group thumbnail carries no objects"))?,
      extra,
      trailing,
    })
  }

  /// Writes the thumbnail to a path, creating the directories leading to it.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be written.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    Ok(StagedFile::write_creating_directories(
      path.as_ref(),
      &self.write_to_bytes::<T>()?,
    )?)
  }

  /// The thumbnail as the bytes of a file.
  ///
  /// # Errors
  ///
  /// Returns an error when the objects outnumber their `u32` count, or the writer refuses the bytes.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    self.write_to::<T>(&mut bytes)?;

    Ok(bytes)
  }

  /// Writes the thumbnail in the order `EGroupThumbnail::Save` writes it, then every chunk kept as read.
  ///
  /// # Errors
  ///
  /// Returns an error when the objects outnumber their `u32` count, or the writer refuses the bytes.
  pub fn write_to<T: ByteOrder>(&self, destination: &mut dyn Write) -> XrfResult {
    let trailing: &[ThmTrailingBytes] = &self.trailing;

    ThmChunks::write::<T>(destination, ThmFile::VERSION_CHUNK_ID, trailing, |writer| {
      Ok(writer.write_u16::<T>(Self::VERSION)?)
    })?;
    ThmChunks::write::<T>(destination, self.thumbnail.to_chunk_id(), trailing, |writer| {
      self.thumbnail.write(writer)
    })?;
    ThmChunks::write::<T>(destination, ThmFile::THUMBNAIL_TYPE_CHUNK_ID, trailing, |writer| {
      Ok(writer.write_u32::<T>(Self::THUMBNAIL_TYPE)?)
    })?;
    ThmChunks::write::<T>(destination, Self::OBJECTS_CHUNK_ID, trailing, |writer| {
      writer.write_u32::<T>(to_format_size(self.objects.len(), "group thumbnail objects")?)?;

      for object in &self.objects {
        writer.write_w1251_string(object)?;
      }

      Ok(())
    })?;

    ThmChunks::write_extra::<T>(destination, &self.extra)
  }
}
