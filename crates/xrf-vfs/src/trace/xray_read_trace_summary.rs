use serde::{Deserialize, Serialize};

/// What a session read, and how much of it was the same bytes twice.
///
/// Reported rather than inferred, for the same reason [`crate::XrayCacheStats`] is: redundancy is invisible to every
/// duration a run already records, and the difference between `bytes` and `unique_bytes` is the whole question.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayReadTraceSummary {
  /// Distinct logical paths read at least once.
  pub paths: usize,
  /// Physical reads performed. Equal to `paths` when nothing was read twice.
  pub reads: u64,
  /// Bytes read in total.
  pub bytes: u64,
  /// Bytes the same work would have cost had every path been read exactly once.
  pub unique_bytes: u64,
  /// The most-read paths, longest first, capped by the caller.
  ///
  /// A slice of `paths`, never all of it: a full sweep touches tens of thousands.
  pub hottest: Vec<XrayReadTraceHotPath>,
}

/// One path that was read more than the others.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayReadTraceHotPath {
  pub path: String,
  pub reads: u64,
  pub bytes: u64,
}
