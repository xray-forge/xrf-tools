use serde::Serialize;
use xrf_translation::TranslationProjectDescriptor;

/// How a save ended, once its edits were on disk.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TranslationSaveOutcome {
  /// The edits are on disk, and this is the project as it now reads.
  // Boxed because the other variant is empty and a descriptor is the whole project: every `Stale` answer would
  // otherwise carry room for a tree it intentionally withholds. Indirection Specta and serde both see through.
  Saved { project: Box<TranslationProjectDescriptor> },
  /// The edits are on disk, but another project replaced this one while they were being written.
  Stale,
}
