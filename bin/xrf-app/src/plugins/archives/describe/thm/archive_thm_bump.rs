use serde::Serialize;
use xrf_thm::ThmBumpChunk;
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// The bump declaration of a descriptor, `THM_CHUNK_BUMP`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmBump {
  pub mode_label: String,
  pub mode: u32,
  /// Height the generator builds the pair against, read at generation time and never at runtime.
  pub virtual_height: f32,
  /// The bump texture named, absent when the chunk names none.
  pub texture: Option<ArchiveReference>,
  /// Whether the engine would try to resolve the name: a mode that uses one, and a name to use.
  pub is_used: bool,
}

impl ArchiveThmBump {
  /// The bump chunk as the viewer reads it, with the texture it names resolved.
  pub fn of(source: &ArchiveDescribeSource, bump: &ThmBumpChunk) -> Self {
    Self {
      mode_label: bump.mode.label(),
      mode: bump.mode.into(),
      virtual_height: bump.virtual_height,
      texture: (!bump.name.is_empty()).then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &bump.name)),
      is_used: bump.is_used(),
    }
  }
}
