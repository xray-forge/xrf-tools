use std::sync::Arc;

use serde::Serialize;
use xrf_translation::TranslationProjectDescriptor;

use crate::core::session::SessionSnapshot;

/// How a save ended, once its edits were on disk.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TranslationSaveOutcome {
  /// The edits are on disk, and this is the project as it now reads.
  // Shares the exact committed snapshot without copying its translation tree.
  Saved {
    project: Arc<SessionSnapshot<TranslationProjectDescriptor>>,
  },
  /// The edits are on disk, but another project replaced this one while they were being written.
  Stale,
}
