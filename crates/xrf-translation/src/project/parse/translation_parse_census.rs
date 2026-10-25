use serde::{Deserialize, Serialize};

/// How much one import run moved.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationParseCensus {
  /// String tables read out of the scope.
  pub files_read: u32,
  /// JSON sources created because nothing was there yet.
  pub files_created: u32,
  /// JSON sources rewritten because merging changed something in them.
  pub files_updated: u32,
  /// JSON sources left alone because merging changed nothing.
  pub files_unchanged: u32,
  /// Files skipped without being read, because they are not string tables or hold no entries.
  pub files_skipped: u32,
  /// Entries read out of the XML, before any merging.
  pub entries_read: u32,
  /// Ids this run introduced to their file.
  pub entries_inserted: u32,
  /// Placeholders this run replaced with text.
  pub entries_filled: u32,
  /// Ids whose text already matched what was read.
  pub entries_unchanged: u32,
  /// Ids whose existing text differed from what was read.
  ///
  /// Kept unless the run was told to overwrite, in which case this counts what was replaced.
  pub entries_conflicted: u32,
  /// `null` placeholders added for languages a file carries but a record did not.
  pub placeholders_added: u32,
}
