use std::time::Duration;

use serde::Serialize;

/// What one initialization run scanned and scaffolded.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationInitializeResult {
  #[serde(with = "xrf_utils::duration_ms")]
  pub duration: Duration,
  /// Sources read, whether or not they needed anything.
  pub files_read: u32,
  /// Sources rewritten because a language was missing from them.
  pub files_initialized: u32,
  /// Files passed over because they are not multi-language JSON sources.
  pub files_skipped: u32,
  /// Placeholders added across every source, one per id and missing language.
  pub keys_added: u32,
}

impl TranslationInitializeResult {
  pub fn new() -> Self {
    Self::default()
  }

  /// Fold one source's totals into the run's.
  pub(crate) fn merge(&mut self, other: &Self) {
    self.files_read += other.files_read;
    self.files_initialized += other.files_initialized;
    self.files_skipped += other.files_skipped;
    self.keys_added += other.keys_added;
  }
}
