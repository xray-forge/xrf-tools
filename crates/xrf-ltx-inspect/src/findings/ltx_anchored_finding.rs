use serde::Serialize;
use xrf_ltx::LtxFinding;

/// One finding, placed at the line a person has to open.
///
/// The finding itself is the core's: this adds only the line, which takes the documents to find. Flattened, so a
/// viewer reads one record rather than a finding nested in its placement.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxAnchoredFinding {
  #[serde(flatten)]
  pub finding: LtxFinding,
  /// One-based line in the finding's `file`, where the anchor reached one.
  pub line: Option<u32>,
}
