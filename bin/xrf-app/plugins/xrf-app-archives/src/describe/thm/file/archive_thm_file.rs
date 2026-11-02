use serde::Serialize;
use xrf_thm::ThmFile;

use crate::describe::thm::file::archive_thm_thumbnail::ArchiveThmThumbnail;

/// What the file is, apart from what it declares.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmFile {
  pub version: Option<u16>,
  /// Whether the version is the one `ETextureThumbnail::Load` accepts.
  pub is_supported_version: bool,
  /// The kind of asset the thumbnail describes; `1` is a texture and the only kind this describer reads.
  pub thumbnail_type: Option<u32>,
  pub thumbnail: Option<ArchiveThmThumbnail>,
  /// Chunk ids the reader could not fold into a field, in the order it read them.
  pub extra_chunks: Vec<u32>,
  /// Chunk ids holding bytes past their fields, which no engine reads, in the order the reader met them.
  pub trailing_chunks: Vec<u32>,
}

impl ArchiveThmFile {
  pub fn of(file: &ThmFile) -> Self {
    Self {
      version: file.version,
      is_supported_version: file.is_supported_version(),
      thumbnail_type: file.thumbnail_type,
      thumbnail: file.thumbnail.as_ref().map(ArchiveThmThumbnail::of),
      extra_chunks: file.extra.iter().map(|chunk| chunk.id).collect(),
      trailing_chunks: file.trailing.iter().map(|trailing| trailing.id).collect(),
    }
  }
}
