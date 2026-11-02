use serde::Serialize;
use xrf_thm::{ThmDetailChunk, ThmDetailUsage};
use xrf_vfs::XrayAssetType;

use crate::describe::description::ArchiveReference;
use crate::describe::source::ArchiveDescribeSource;

/// The detail association of a descriptor, `THM_CHUNK_DETAIL_EXT`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmDetail {
  pub scale: f32,
  /// The detail texture named, absent when the chunk names none.
  pub texture: Option<ArchiveReference>,
  /// The flags that switch this association on, by the SDK's own spelling.
  pub enabled_by: Vec<String>,
}

impl ArchiveThmDetail {
  /// The detail chunk as the viewer reads it, with the texture it names resolved.
  ///
  /// `usage` is [`ThmFile::used_detail_usage`](xrf_thm::ThmFile::used_detail_usage)'s answer rather than a second
  /// reading of the flag word: which flags make an association live is engine behavior, and `xrf-thm` owns it.
  pub fn of(source: &ArchiveDescribeSource, detail: &ThmDetailChunk, usage: Option<ThmDetailUsage>) -> Self {
    Self {
      scale: detail.scale,
      texture: (!detail.name.is_empty()).then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &detail.name)),
      enabled_by: usage
        .map_or(&[][..], ThmDetailUsage::get_flags)
        .iter()
        .map(|flag| flag.label().to_owned())
        .collect(),
    }
  }
}
