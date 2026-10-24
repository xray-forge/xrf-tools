use serde::Serialize;
use xrf_job::JobOutcome;

/// How a level map's or cubemap's capture ended: written whole, or stopped when asked, writing nothing.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureOutcome {
  pub outcome: JobOutcome,
  /// The picture's pixels across and down, as written: a map's, or a cubemap's cross.
  pub width: u32,
  pub height: u32,
}

impl CaptureOutcome {
  /// A capture stopped when asked, which wrote nothing.
  pub fn cancelled() -> Self {
    Self {
      outcome: JobOutcome::Cancelled,
      width: 0,
      height: 0,
    }
  }
}
