use serde::Serialize;
use xrf_skeleton::SkeletonMotionMark;

use crate::describe::omf::archive_omf_mark_interval::ArchiveOmfMarkInterval;

/// One named set of moments within a motion, `motion_marks`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfMark {
  pub name: String,
  /// The moments the mark covers, in the order the file lists them.
  pub intervals: Vec<ArchiveOmfMarkInterval>,
}

impl ArchiveOmfMark {
  pub fn of(mark: &SkeletonMotionMark) -> Self {
    Self {
      name: mark.name.clone(),
      intervals: mark
        .intervals
        .iter()
        .map(|(from, to)| ArchiveOmfMarkInterval { from: *from, to: *to })
        .collect(),
    }
  }
}
