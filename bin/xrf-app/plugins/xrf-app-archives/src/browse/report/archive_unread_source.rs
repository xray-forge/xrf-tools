use std::path::PathBuf;

use serde::Serialize;
use xrf_vfs::XraySkippedMount;

/// A source the plan named that could not be opened, so the search reaches nothing it holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveUnreadSource {
  /// Where the source that failed to open lives.
  pub path: PathBuf,
  /// How the plan described it, such as an `fsgame.ltx` alias.
  pub origin: String,
  /// Why it could not be opened.
  pub reason: String,
}

impl From<&XraySkippedMount> for ArchiveUnreadSource {
  fn from(skipped: &XraySkippedMount) -> Self {
    Self {
      origin: skipped.origin.clone(),
      path: skipped.path.clone(),
      reason: skipped.reason.clone(),
    }
  }
}
