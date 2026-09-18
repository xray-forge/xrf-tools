use serde::Serialize;
use xrf_ppe::PpeColorMap;
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::animation::ArchiveAnimationChannel;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// The name the influence channel is described under, which no part of the file spells.
const INFLUENCE: &str = "colour map influence";

/// The colour grading an effect applies, which only version 2 carries.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePpeColorMap {
  /// The gradient texture the grading samples, absent when the effect names none.
  pub texture: Option<ArchiveReference>,
  /// How much of the graded colour is mixed in over time.
  pub influence: ArchiveAnimationChannel,
  /// Whether the effect grades at all, which is a name being present rather than an influence being non-zero.
  pub is_used: bool,
}

impl ArchivePpeColorMap {
  /// The grading as the viewer reads it, with the gradient it names resolved.
  pub fn of(source: &ArchiveDescribeSource, color_map: &PpeColorMap) -> Self {
    Self {
      texture: (!color_map.texture.is_empty())
        .then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &color_map.texture)),
      influence: ArchiveAnimationChannel::of(INFLUENCE, &color_map.influence),
      is_used: color_map.is_used(),
    }
  }
}
