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
use crate::chunks::thm_trailing_bytes::ThmTrailingBytes;
use crate::thm_file::ThmFile;

/// A sound's thumbnail, `ESoundThumbnail` (`ECore/Editor/EThumbnailSound.cpp`): the `.thm` beside a source `.wav` under
/// the SDK's `$sounds$`, holding what `CSoundManager::MakeGameSound` encodes the game `.ogg` with. Only the SDK reads
/// it; the game reads the same parameters from the `.ogg`'s own comment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmSoundFile {
  /// `m_fQuality`, the Vorbis quality the game sound is encoded at.
  pub quality: f32,
  /// Metres within which the sound plays at full volume.
  pub min_distance: f32,
  /// Metres past which the sound is not heard.
  pub max_distance: f32,
  /// What the sound means to the AI, an `ESoundTypes` mask.
  pub game_type: u32,
  /// `THM_CHUNK_SOUNDPARAM2`, which a thumbnail older than it lacks; see [`Self::get_base_volume`].
  pub base_volume: Option<f32>,
  /// `THM_CHUNK_SOUND_AI_DIST`, which a thumbnail older than it lacks; see [`Self::get_max_ai_distance`].
  pub max_ai_distance: Option<f32>,
  /// What the reader could not fold into a field; see [`ThmExtraChunk`].
  pub extra: Vec<ThmExtraChunk>,
  /// What folded chunks hold past their fields; see [`ThmTrailingBytes`].
  pub trailing: Vec<ThmTrailingBytes>,
}

impl ThmSoundFile {
  /// `THM_SOUND_VERSION`, the only version `ESoundThumbnail::Load` accepts.
  pub const VERSION: u16 = 0x0014;

  /// `ECustomThumbnail::ETSound`.
  pub const THUMBNAIL_TYPE: u32 = 2;

  /// `THM_CHUNK_SOUNDPARAM`: quality, minimum and maximum distance, game type.
  pub const PARAMETERS_CHUNK_ID: u32 = 0x1000;

  /// `THM_CHUNK_SOUNDPARAM2`: base volume.
  pub const BASE_VOLUME_CHUNK_ID: u32 = 0x1001;

  /// `THM_CHUNK_SOUND_AI_DIST`: maximum AI distance.
  pub const MAX_AI_DISTANCE_CHUNK_ID: u32 = 0x1002;

  /// The base volume a thumbnail without the chunk reads as, the constructor's.
  pub const DEFAULT_BASE_VOLUME: f32 = 1.0;

  /// A thumbnail for a sound that has none, with the values `ESoundThumbnail`'s constructor starts one at.
  pub fn new_sound() -> Self {
    Self {
      quality: 0.0,
      min_distance: 1.0,
      max_distance: 300.0,
      game_type: 0,
      base_volume: Some(Self::DEFAULT_BASE_VOLUME),
      max_ai_distance: Some(300.0),
      extra: Vec::new(),
      trailing: Vec::new(),
    }
  }

  /// The base volume the SDK encodes the sound with: the stored one, or 1 for a thumbnail without it.
  pub fn get_base_volume(&self) -> f32 {
    self.base_volume.unwrap_or(Self::DEFAULT_BASE_VOLUME)
  }

  /// The maximum AI distance the SDK encodes the sound with: the stored one, or the maximum distance for a thumbnail
  /// without it, as `ESoundThumbnail::Load` reads it.
  pub fn get_max_ai_distance(&self) -> f32 {
    self.max_ai_distance.unwrap_or(self.max_distance)
  }
}

impl ThmSoundFile {
  /// Reads a sound thumbnail from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a sound thumbnail [`Self::read_from_chunk`] reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Sound thumbnail was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)?)
  }

  /// Reads a sound thumbnail from bytes already in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound thumbnail [`Self::read_from_chunk`] reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads a sound thumbnail as `ESoundThumbnail::Load` does.
  ///
  /// # Errors
  ///
  /// Returns an error, where the SDK fails or asserts, for a version other than [`Self::VERSION`], a type other than
  /// [`Self::THUMBNAIL_TYPE`], a missing parameter chunk, or a chunk too short for its fields.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut thumbnail_type: Option<u32> = None;
    let mut parameters: Option<(f32, f32, f32, u32)> = None;
    let mut file: Self = Self {
      base_volume: None,
      max_ai_distance: None,
      ..Self::new_sound()
    };

    for mut chunk in reader.read_children()? {
      let is_folded: bool = match chunk.id {
        ThmFile::VERSION_CHUNK_ID if version.is_none() => {
          version = Some(chunk.read_u16::<T>()?);
          true
        }
        ThmFile::THUMBNAIL_TYPE_CHUNK_ID if thumbnail_type.is_none() => {
          thumbnail_type = Some(chunk.read_u32::<T>()?);
          true
        }
        Self::PARAMETERS_CHUNK_ID if parameters.is_none() => {
          parameters = Some((
            chunk.read_f32::<T>()?,
            chunk.read_f32::<T>()?,
            chunk.read_f32::<T>()?,
            chunk.read_u32::<T>()?,
          ));
          true
        }
        Self::BASE_VOLUME_CHUNK_ID if file.base_volume.is_none() => {
          file.base_volume = Some(chunk.read_f32::<T>()?);
          true
        }
        Self::MAX_AI_DISTANCE_CHUNK_ID if file.max_ai_distance.is_none() => {
          file.max_ai_distance = Some(chunk.read_f32::<T>()?);
          true
        }
        _ => false,
      };

      ThmChunks::keep_rest(&mut chunk, is_folded, &mut file.extra, &mut file.trailing)?;
    }

    ThmChunks::require_kind("a sound", version, Self::VERSION, thumbnail_type, Self::THUMBNAIL_TYPE)?;

    let (quality, min_distance, max_distance, game_type) =
      parameters.ok_or_else(|| XrfError::new_parsing_error("Sound thumbnail carries no sound parameters"))?;

    Ok(Self {
      quality,
      min_distance,
      max_distance,
      game_type,
      ..file
    })
  }
}

impl ThmSoundFile {
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

  /// Writes the thumbnail in the order `ESoundThumbnail::Save` writes it, the base volume and maximum AI distance only
  /// where the thumbnail carries them, then every chunk kept as read.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write_to<T: ByteOrder>(&self, destination: &mut dyn Write) -> XrfResult {
    let trailing: &[ThmTrailingBytes] = &self.trailing;

    ThmChunks::write::<T>(destination, ThmFile::VERSION_CHUNK_ID, trailing, |writer| {
      Ok(writer.write_u16::<T>(Self::VERSION)?)
    })?;
    ThmChunks::write::<T>(destination, ThmFile::THUMBNAIL_TYPE_CHUNK_ID, trailing, |writer| {
      Ok(writer.write_u32::<T>(Self::THUMBNAIL_TYPE)?)
    })?;
    ThmChunks::write::<T>(destination, Self::PARAMETERS_CHUNK_ID, trailing, |writer| {
      writer.write_f32::<T>(self.quality)?;
      writer.write_f32::<T>(self.min_distance)?;
      writer.write_f32::<T>(self.max_distance)?;
      writer.write_u32::<T>(self.game_type)?;

      Ok(())
    })?;

    if let Some(base_volume) = self.base_volume {
      ThmChunks::write::<T>(destination, Self::BASE_VOLUME_CHUNK_ID, trailing, |writer| {
        Ok(writer.write_f32::<T>(base_volume)?)
      })?;
    }

    if let Some(max_ai_distance) = self.max_ai_distance {
      ThmChunks::write::<T>(destination, Self::MAX_AI_DISTANCE_CHUNK_ID, trailing, |writer| {
        Ok(writer.write_f32::<T>(max_ai_distance)?)
      })?;
    }

    ThmChunks::write_extra::<T>(destination, &self.extra)
  }
}
