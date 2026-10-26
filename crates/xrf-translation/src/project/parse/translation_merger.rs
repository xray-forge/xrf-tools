use std::collections::BTreeSet;

use crate::json::translation_json_canon::TranslationJsonCanon;
use crate::project::parse::translation_merge_outcome::TranslationMergeOutcome;
use crate::text::translation_entry::TranslationEntry;
use crate::text::translation_json::TranslationJson;
use crate::text::translation_variant::TranslationVariant;

/// Merges one language's string table into the JSON source a target file already holds.
pub(crate) struct TranslationMerger;

impl TranslationMerger {
  /// Merge one language's entries into what a source already holds.
  ///
  /// `existing` is what was on disk, or empty for a file that is not there yet. The merge is additive: an id the XML
  /// no longer carries keeps whatever the JSON says, because this imports a language, it does not mirror a tree.
  ///
  /// Existing text that differs from what was read is **kept** unless `is_overwrite`, and counted either way. A
  /// translator's edits are the thing most likely to differ from the mod they were imported from, and a merge that
  /// silently replaced them would destroy exactly the work this format exists to hold.
  pub(crate) fn merge(
    existing: TranslationJson,
    incoming: &[(String, String)],
    language: &str,
    is_overwrite: bool,
  ) -> (TranslationJson, TranslationMergeOutcome) {
    let mut merged: TranslationJson = existing;
    let mut outcome: TranslationMergeOutcome = TranslationMergeOutcome::default();

    for (id, text) in incoming {
      let value: TranslationVariant = TranslationVariant::from_table_text(text);
      let entry: &mut TranslationEntry = merged.entry(id.clone()).or_default();

      match entry.get(language) {
        // Nothing there at all: the id is new to this file, or the file never carried this language.
        None => {
          outcome.inserted += 1;
          entry.insert(language.to_owned(), Some(value));
        }
        // A placeholder waiting for a translator, which is what this run exists to fill.
        Some(None) => {
          outcome.filled += 1;
          entry.insert(language.to_owned(), Some(value));
        }
        Some(Some(present)) => {
          // Compared as the single line a string table holds, so a hand-authored array and an imported string that
          // mean the same text are the same text. Without this every re-import of a multi-line description would
          // report a conflict against the value it wrote itself.
          if present.to_single_line() == value.to_single_line() {
            outcome.unchanged += 1;
          } else {
            outcome.conflicted += 1;

            if is_overwrite {
              entry.insert(language.to_owned(), Some(value));
            }
          }
        }
      }
    }

    outcome.placeholders_added = Self::add_placeholders(&mut merged);

    // Whether this changed the file is not decided here. The caller renders the document through the canonical writer
    // and compares bytes, which also catches a file whose records were already right and whose formatting was not.
    TranslationJsonCanon::sort(&mut merged);

    (merged, outcome)
  }

  /// Give every record an explicit `null` for each language the file carries but it does not have.
  ///
  /// The languages of the file, not the eight this crate knows: importing an English and Russian mod should not declare
  /// six more languages missing that were never in scope. `initialize` is the command that scaffolds all eight, for a
  /// project that does intend to ship them.
  fn add_placeholders(document: &mut TranslationJson) -> u32 {
    let languages: BTreeSet<String> = document
      .values()
      .flat_map(|entry| entry.keys().cloned())
      .collect::<BTreeSet<String>>();

    let mut added: u32 = 0;

    for entry in document.values_mut() {
      for language in &languages {
        if !entry.contains_key(language) {
          added += 1;
          entry.insert(language.clone(), None);
        }
      }
    }

    added
  }
}
