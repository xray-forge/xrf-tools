use std::collections::BTreeSet;

use crate::project::parse::result::ProjectParseCensus;
use crate::types::{TranslationEntry, TranslationJson, TranslationVariant};

/// The engine reads this literal pair of characters in a string table as a line break.
const ENGINE_LINE_BREAK: &str = "\\n";

/// What merging one string table into one JSON source changed.
#[derive(Debug, Default)]
pub(crate) struct MergeOutcome {
  pub inserted: u32,
  pub filled: u32,
  pub unchanged: u32,
  pub conflicted: u32,
  pub placeholders_added: u32,
  /// Whether the merged document differs from the one that was read.
  ///
  /// Tracked rather than derived from the counts, because sorting alone can change a file that gained
  /// no entry — which is exactly what normalizes a tree somebody hand-edited out of order.
  pub is_changed: bool,
}

impl MergeOutcome {
  pub(crate) fn record(&self, census: &mut ProjectParseCensus) {
    census.entries_inserted += self.inserted;
    census.entries_filled += self.filled;
    census.entries_unchanged += self.unchanged;
    census.entries_conflicted += self.conflicted;
    census.placeholders_added += self.placeholders_added;
  }
}

/// Merge one language's entries into the JSON a target file already holds.
///
/// `existing` is what was on disk, or empty for a file that is not there yet. The merge is additive:
/// an id the XML no longer carries keeps whatever the JSON says, because this imports a language, it
/// does not mirror a tree.
///
/// Existing text that differs from what was read is **kept** unless `is_overwrite`, and counted either
/// way. A translator's edits are the thing most likely to differ from the mod they were imported from,
/// and a merge that silently replaced them would destroy exactly the work this format exists to hold.
pub(crate) fn merge_entries(
  existing: TranslationJson,
  incoming: &[(String, String)],
  language: &str,
  is_overwrite: bool,
) -> (TranslationJson, MergeOutcome) {
  let original: TranslationJson = existing;
  let mut merged: TranslationJson = original.clone();
  let mut outcome: MergeOutcome = MergeOutcome::default();

  for (id, text) in incoming {
    let value: TranslationVariant = to_variant(text);
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
        // Compared as the single line a string table holds, so a hand-authored array and an imported
        // string that mean the same text are the same text. Without this every re-import of a
        // multi-line description would report a conflict against the value it wrote itself.
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

  outcome.placeholders_added = add_placeholders(&mut merged);

  sort_document(&mut merged);

  outcome.is_changed = !is_same_document(&original, &merged);

  (merged, outcome)
}

/// Whether two documents would serialize identically, order included.
///
/// `IndexMap` compares as a map, so `==` answers true for two documents holding the same pairs in
/// different orders — which is exactly the change a re-sort makes and nothing else does. Using `==`
/// here meant a file somebody hand-edited out of order was detected as unchanged and left that way.
fn is_same_document(left: &TranslationJson, right: &TranslationJson) -> bool {
  left.len() == right.len()
    && left
      .iter()
      .zip(right.iter())
      .all(|((left_id, left_entry), (right_id, right_entry))| {
        left_id == right_id && is_same_entry(left_entry, right_entry)
      })
}

fn is_same_entry(left: &TranslationEntry, right: &TranslationEntry) -> bool {
  left.len() == right.len()
    && left
      .iter()
      .zip(right.iter())
      .all(|((left_language, left_value), (right_language, right_value))| {
        left_language == right_language && left_value == right_value
      })
}

/// Give every record an explicit `null` for each language the file carries but it does not have.
///
/// The languages of the file, not the eight this crate knows: importing an English and Russian mod
/// should not declare six more languages missing that were never in scope. `initialize` is the command
/// that scaffolds all eight, for a project that does intend to ship them.
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

/// Sort ids and language keys, so output does not depend on the order the languages were run in.
///
/// The whole workflow is "run once per language and merge", so a file whose shape remembered which
/// run touched it first would churn a diff for nothing. It also matches every hand-authored source in
/// the engine, which are sorted both ways already.
fn sort_document(document: &mut TranslationJson) {
  document.sort_keys();

  for entry in document.values_mut() {
    entry.sort_keys();
  }
}

/// Read one string table value as the JSON form that keeps its structure.
///
/// Splitting is lossless: `build` joins a multi-line value back on the same literal, so the text that
/// reaches the game is byte for byte what was imported. What it buys is a description that can be read
/// and reviewed in the source instead of being one 400-character line.
fn to_variant(text: &str) -> TranslationVariant {
  if text.contains(ENGINE_LINE_BREAK) {
    return TranslationVariant::MultiString(text.split(ENGINE_LINE_BREAK).map(String::from).collect());
  }

  TranslationVariant::String(text.to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn document(source: &str) -> TranslationJson {
    serde_json::from_str(source).expect("Expected valid test JSON")
  }

  #[test]
  fn an_absent_id_is_inserted() {
    let (merged, outcome) = merge_entries(
      TranslationJson::default(),
      &[(String::from("st_a"), String::from("A"))],
      "eng",
      false,
    );

    assert_eq!(outcome.inserted, 1);
    assert!(outcome.is_changed);
    assert_eq!(
      merged["st_a"]["eng"],
      Some(TranslationVariant::String(String::from("A")))
    );
  }

  #[test]
  fn a_null_placeholder_is_filled() {
    let (merged, outcome) = merge_entries(
      document(r#"{"st_a":{"eng":null}}"#),
      &[(String::from("st_a"), String::from("A"))],
      "eng",
      false,
    );

    assert_eq!(outcome.filled, 1);
    assert_eq!(outcome.inserted, 0);
    assert_eq!(
      merged["st_a"]["eng"],
      Some(TranslationVariant::String(String::from("A")))
    );
  }

  #[test]
  fn existing_text_survives_a_conflict_unless_overwrite_is_asked_for() {
    let kept = merge_entries(
      document(r#"{"st_a":{"eng":"Mine"}}"#),
      &[(String::from("st_a"), String::from("Theirs"))],
      "eng",
      false,
    );

    assert_eq!(kept.1.conflicted, 1);
    assert!(!kept.1.is_changed);
    assert_eq!(
      kept.0["st_a"]["eng"],
      Some(TranslationVariant::String(String::from("Mine")))
    );

    let replaced = merge_entries(
      document(r#"{"st_a":{"eng":"Mine"}}"#),
      &[(String::from("st_a"), String::from("Theirs"))],
      "eng",
      true,
    );

    assert_eq!(replaced.1.conflicted, 1);
    assert!(replaced.1.is_changed);
    assert_eq!(
      replaced.0["st_a"]["eng"],
      Some(TranslationVariant::String(String::from("Theirs")))
    );
  }

  #[test]
  fn an_array_and_the_string_it_joins_to_are_the_same_text() {
    // Re-importing what a previous run wrote must not read as a conflict, or every multi-line
    // description in the tree would report one on every run.
    let (_, outcome) = merge_entries(
      document(r#"{"st_a":{"eng":["first","second"]}}"#),
      &[(String::from("st_a"), String::from("first\\nsecond"))],
      "eng",
      false,
    );

    assert_eq!(outcome.unchanged, 1);
    assert_eq!(outcome.conflicted, 0);
  }

  #[test]
  fn multi_line_text_is_split_into_an_array() {
    let (merged, _) = merge_entries(
      TranslationJson::default(),
      &[(String::from("st_a"), String::from("first\\nsecond"))],
      "eng",
      false,
    );

    assert_eq!(
      merged["st_a"]["eng"],
      Some(TranslationVariant::MultiString(vec![
        String::from("first"),
        String::from("second")
      ]))
    );
  }

  #[test]
  fn every_record_gains_a_placeholder_for_the_languages_the_file_carries() {
    // `st_b` has no Russian, and the file has Russian, so the gap becomes visible rather than absent.
    let (merged, outcome) = merge_entries(
      document(r#"{"st_a":{"eng":"A","rus":"А"}}"#),
      &[(String::from("st_b"), String::from("B"))],
      "eng",
      false,
    );

    assert_eq!(outcome.placeholders_added, 1);
    assert!(merged["st_b"].contains_key("rus"));
    assert_eq!(merged["st_b"]["rus"], None);
    // No language the file does not carry is invented.
    assert!(!merged["st_b"].contains_key("ukr"));
  }

  #[test]
  fn output_does_not_depend_on_the_order_the_languages_were_run_in() {
    let english = &[(String::from("st_b"), String::from("B"))];
    let russian = &[(String::from("st_a"), String::from("А"))];

    let (first, _) = merge_entries(TranslationJson::default(), english, "eng", false);
    let (english_then_russian, _) = merge_entries(first, russian, "rus", false);

    let (second, _) = merge_entries(TranslationJson::default(), russian, "rus", false);
    let (russian_then_english, _) = merge_entries(second, english, "eng", false);

    assert_eq!(
      serde_json::to_string(&english_then_russian).unwrap(),
      serde_json::to_string(&russian_then_english).unwrap()
    );
  }

  #[test]
  fn sorting_alone_counts_as_a_change() {
    // A tree somebody added records to by hand is normalized by the next run that touches the file,
    // even when that run merges nothing new into it.
    let (merged, outcome) = merge_entries(
      document(r#"{"st_b":{"eng":"B"},"st_a":{"eng":"A"}}"#),
      &[],
      "eng",
      false,
    );

    assert!(outcome.is_changed);
    assert_eq!(merged.keys().collect::<Vec<_>>(), vec!["st_a", "st_b"]);
  }

  #[test]
  fn an_already_sorted_and_complete_file_is_left_alone() {
    let (_, outcome) = merge_entries(
      document(r#"{"st_a":{"eng":"A"}}"#),
      &[(String::from("st_a"), String::from("A"))],
      "eng",
      false,
    );

    assert!(!outcome.is_changed);
    assert_eq!(outcome.unchanged, 1);
  }
}
