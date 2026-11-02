use serde::Serialize;
use xrf_thm::ThmThumbnailChunk;

/// The preview picture a descriptor carries, `THM_CHUNK_DATA`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmThumbnail {
  /// Whether the payload is the engine's own compressed stream, which is the only form seen in the wild.
  pub is_compressed: bool,
  pub size: u64,
}

impl ArchiveThmThumbnail {
  pub fn of(thumbnail: &ThmThumbnailChunk) -> Self {
    Self {
      is_compressed: thumbnail.is_compressed,
      size: thumbnail.data.len() as u64,
    }
  }
}
