use serde::Serialize;
use xrf_thm::{ThmDetailChunk, ThmDetailUsage, ThmTextureFlag};
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

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
  /// reading of the flag word: which flags make an association live is engine behavior, and `xrf-db` owns it.
  pub fn of(source: &ArchiveDescribeSource, detail: &ThmDetailChunk, usage: Option<ThmDetailUsage>) -> Self {
    Self {
      scale: detail.scale,
      texture: (!detail.name.is_empty()).then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &detail.name)),
      enabled_by: to_enabling_flags(usage),
    }
  }
}

/// The SDK spellings of the flags a live detail association is applied by.
fn to_enabling_flags(usage: Option<ThmDetailUsage>) -> Vec<String> {
  let flags: &[ThmTextureFlag] = match usage {
    Some(ThmDetailUsage::Diffuse) => &[ThmTextureFlag::DiffuseDetail],
    Some(ThmDetailUsage::Bump) => &[ThmTextureFlag::BumpDetail],
    Some(ThmDetailUsage::DiffuseAndBump) => &[ThmTextureFlag::DiffuseDetail, ThmTextureFlag::BumpDetail],
    None => &[],
  };

  flags.iter().map(|flag| flag.label().to_owned()).collect()
}

#[cfg(test)]
mod tests {
  use xrf_thm::ThmDetailUsage;

  use super::to_enabling_flags;

  #[test]
  fn each_usage_names_the_flags_that_produce_it() {
    assert_eq!(
      to_enabling_flags(Some(ThmDetailUsage::Diffuse)),
      vec!["flDiffuseDetail"]
    );
    assert_eq!(to_enabling_flags(Some(ThmDetailUsage::Bump)), vec!["flBumpDetail"]);
    assert_eq!(
      to_enabling_flags(Some(ThmDetailUsage::DiffuseAndBump)),
      vec!["flDiffuseDetail", "flBumpDetail"]
    );
  }

  #[test]
  fn an_association_the_engine_reads_past_names_nothing() {
    assert!(to_enabling_flags(None).is_empty());
  }
}
