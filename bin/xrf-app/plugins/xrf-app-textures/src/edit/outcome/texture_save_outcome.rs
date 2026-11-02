use serde::Serialize;
use xrf_job::JobOutcome;

/// What a save left on disk.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureSaveOutcome {
  /// Whether the save wrote what it was asked to or stopped because it was asked to.
  pub outcome: JobOutcome,
  /// The files written, in the order they were written.
  pub written: Vec<String>,
}
