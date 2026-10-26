use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

use crate::object::thm_group_file::ThmGroupFile;
use crate::object::thm_object_file::ThmObjectFile;
use crate::sound::thm_sound_file::ThmSoundFile;
use crate::thm_file::ThmFile;

/// What a `.thm` describes, which decides the reader it takes: the SDK's `ECustomThumbnail::THMType` with its version,
/// since a group's thumbnail carries an object's type and the two share chunk ids that mean different things.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThmKind {
  /// A texture descriptor, [`ThmFile`], which the engine reads too.
  Texture,
  /// An object's thumbnail, [`ThmObjectFile`].
  Object,
  /// A group's thumbnail, [`ThmGroupFile`].
  Group,
  /// A sound's thumbnail, [`ThmSoundFile`].
  Sound,
}

impl ThmKind {
  /// Every kind, in the order `THMType` numbers them, the group after the object it is marked as.
  pub const ALL: [Self; 4] = [Self::Object, Self::Group, Self::Texture, Self::Sound];

  /// The version and type a thumbnail of this kind declares.
  pub const fn get_signature(self) -> (u16, u32) {
    match self {
      Self::Texture => (ThmFile::VERSION, ThmFile::THUMBNAIL_TYPE_TEXTURE),
      Self::Object => (ThmObjectFile::VERSION, ThmObjectFile::THUMBNAIL_TYPE),
      Self::Group => (ThmGroupFile::VERSION, ThmGroupFile::THUMBNAIL_TYPE),
      Self::Sound => (ThmSoundFile::VERSION, ThmSoundFile::THUMBNAIL_TYPE),
    }
  }

  /// The kind a thumbnail's version and type chunks declare, or none for a pair no SDK reader accepts or a file lacking
  /// either.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a chunk tree, or a version or type chunk is too short.
  pub fn detect<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Option<Self>> {
    Self::detect_in::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// [`Self::detect`] over a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a version or type chunk is too short.
  pub fn detect_in<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Option<Self>> {
    let mut version: Option<u16> = None;
    let mut thumbnail_type: Option<u32> = None;

    for mut chunk in reader.read_children_including_compressed()? {
      match chunk.id {
        ThmFile::VERSION_CHUNK_ID if version.is_none() => version = Some(chunk.read_u16::<T>()?),
        ThmFile::THUMBNAIL_TYPE_CHUNK_ID if thumbnail_type.is_none() => thumbnail_type = Some(chunk.read_u32::<T>()?),
        _ => {}
      }
    }

    let Some(signature) = version.zip(thumbnail_type) else {
      return Ok(None);
    };

    Ok(Self::ALL.into_iter().find(|kind| kind.get_signature() == signature))
  }
}
