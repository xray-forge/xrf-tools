use serde::Serialize;
use xrf_db::ThmFile;

/// What the file is, apart from what it declares.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmFile {
  pub version: Option<u16>,
  /// Whether the version is the one `ETextureThumbnail::Load` accepts.
  pub is_supported_version: bool,
  /// The kind of asset the thumbnail describes; `1` is a texture and the only one these tools read.
  pub thumbnail_type: Option<u32>,
  pub thumbnail: Option<ArchiveThmThumbnail>,
  /// Chunk ids the reader could not fold into a field, in the order it read them.
  pub extra_chunks: Vec<u32>,
}

/// The preview picture a descriptor carries, `THM_CHUNK_DATA`.
///
/// Reported by size and never decoded. The trunk SDK stopped writing the chunk - its `w_chunk` call is commented out
/// in `ETextureThumbnail::Save` - and 11 of the 19,849 descriptors across the workspace trees still carry one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmThumbnail {
  /// Whether the payload is the engine's own compressed stream, which is the only form seen in the wild.
  pub is_compressed: bool,
  pub size: u64,
}

impl ArchiveThmFile {
  pub fn of(file: &ThmFile) -> Self {
    Self {
      version: file.version,
      is_supported_version: file.is_supported_version(),
      thumbnail_type: file.thumbnail_type,
      thumbnail: file.thumbnail.as_ref().map(|thumbnail| ArchiveThmThumbnail {
        is_compressed: thumbnail.is_compressed,
        size: thumbnail.data.len() as u64,
      }),
      extra_chunks: file.extra.iter().map(|chunk| chunk.id).collect(),
    }
  }
}
