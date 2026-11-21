use serde::Serialize;
use xrf_job::JobOutcome;

/// What a Save as wrote.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextureSaveAsOutcome {
  /// Whether the copy was written, or stopped on request with nothing written.
  pub outcome: JobOutcome,
  /// The copy's texture name, which the editor opens next.
  pub reference: String,
  /// The files written, by path on disk, in write order.
  pub written: Vec<String>,
}
