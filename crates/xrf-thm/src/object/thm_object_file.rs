use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::chunks::thm_chunks::ThmChunks;
use crate::chunks::thm_extra_chunk::ThmExtraChunk;
use crate::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
use crate::chunks::thm_trailing_bytes::ThmTrailingBytes;
use crate::thm_file::ThmFile;

/// An object's thumbnail, `EObjectThumbnail` (`ECore/Editor/EThumbnailObject.cpp`): the `.thm` beside an `.object`
/// under the SDK's `$objects$`, its preview picture and size. Only the SDK reads it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmObjectFile {
  /// The preview the object library shows, see [`ThmThumbnailChunk::decode`].
  pub thumbnail: ThmThumbnailChunk,
  pub face_count: u32,
  pub vertex_count: u32,
  /// What the reader could not fold into a field; see [`ThmExtraChunk`].
  pub extra: Vec<ThmExtraChunk>,
  /// What folded chunks hold past their fields; see [`ThmTrailingBytes`].
  pub trailing: Vec<ThmTrailingBytes>,
}

impl ThmObjectFile {
  /// `THM_OBJECT_VERSION`, the only version `EObjectThumbnail::Load` accepts.
  pub const VERSION: u16 = 0x0012;

  /// `ECustomThumbnail::ETObject`.
  pub const THUMBNAIL_TYPE: u32 = 0;

  /// `THM_CHUNK_OBJECTPARAM`: face and vertex count. The same id is a texture descriptor's material chunk, which is why
  /// a thumbnail is read for its kind first ([`crate::ThmKind`]).
  pub const PARAMETERS_CHUNK_ID: u32 = 0x0816;

  /// Reads an object thumbnail from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not an object thumbnail [`Self::read_from_chunk`] reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Object thumbnail was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)?)
  }

  /// Reads an object thumbnail from bytes already in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an object thumbnail [`Self::read_from_chunk`] reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads an object thumbnail as `EObjectThumbnail::Load` does.
  ///
  /// # Errors
  ///
  /// Returns an error, where the SDK fails or asserts, for a version other than [`Self::VERSION`], a type other than
  /// [`Self::THUMBNAIL_TYPE`], a missing preview or parameter chunk, or a chunk too short for its fields.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut thumbnail_type: Option<u32> = None;
    let mut thumbnail: Option<ThmThumbnailChunk> = None;
    let mut counts: Option<(u32, u32)> = None;
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
        Self::PARAMETERS_CHUNK_ID if counts.is_none() => {
          counts = Some((chunk.read_u32::<T>()?, chunk.read_u32::<T>()?));
          true
        }
        _ => false,
      };

      ThmChunks::keep_rest(&mut chunk, is_folded, &mut extra, &mut trailing)?;
    }

    ThmChunks::require_kind(
      "an object",
      version,
      Self::VERSION,
      thumbnail_type,
      Self::THUMBNAIL_TYPE,
    )?;

    let (face_count, vertex_count) =
      counts.ok_or_else(|| XrfError::new_parsing_error("Object thumbnail carries no face and vertex counts"))?;

    Ok(Self {
      thumbnail: thumbnail.ok_or_else(|| XrfError::new_parsing_error("Object thumbnail carries no preview"))?,
      face_count,
      vertex_count,
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
  /// Returns an error when the writer refuses the bytes.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    self.write_to::<T>(&mut bytes)?;

    Ok(bytes)
  }

  /// Writes the thumbnail in the order `EObjectThumbnail::Save` writes it, then every chunk kept as read.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
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
    ThmChunks::write::<T>(destination, Self::PARAMETERS_CHUNK_ID, trailing, |writer| {
      writer.write_u32::<T>(self.face_count)?;
      writer.write_u32::<T>(self.vertex_count)?;

      Ok(())
    })?;

    ThmChunks::write_extra::<T>(destination, &self.extra)
  }
}
