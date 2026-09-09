use serde::Serialize;

use crate::pack::ArchivePackResult;

/// Patch publication outcome, serialized with a `kind` tag.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchivePatchPublication {
  /// No write was attempted: a comparison or a run cancelled before publication.
  Compared,
  /// No added or modified entries required publication. Removed entries may still exist.
  Unnecessary,
  /// Publication was attempted; the result records completion, cancellation, and retained volumes.
  Published(ArchivePackResult),
}

impl ArchivePatchPublication {
  /// Returns the publication result, if publication was attempted.
  pub fn get_published(&self) -> Option<&ArchivePackResult> {
    match self {
      Self::Published(result) => Some(result),
      Self::Compared | Self::Unnecessary => None,
    }
  }

  /// Whether publication was attempted, including a cancelled write.
  pub fn is_published(&self) -> bool {
    matches!(self, Self::Published(_))
  }
}
