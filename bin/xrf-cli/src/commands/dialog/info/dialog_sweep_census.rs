use std::collections::BTreeMap;

use serde::Serialize;

/// What a sweep counted, beside what it found.
///
/// These are the numbers the reader's premises rest on: that phrases nearly always sit in a
/// `phrase_list`, that a dialog may legitimately declare none, and that the element set is the one
/// `DialogElementKind` classifies.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogSweepCensus {
  pub files: usize,
  pub unreadable_files: usize,
  pub archived_files: usize,
  pub dialogs: usize,
  pub phrases: usize,
  pub dialogs_without_phrases: usize,
  pub dialogs_with_priority: usize,
  pub phrases_outside_phrase_list: usize,
  pub phrases_without_text: usize,
  pub final_phrases: usize,
  pub links: usize,
  pub largest_dialog_phrases: usize,
  pub largest_dialog_id: Option<String>,
  pub encodings: BTreeMap<String, usize>,
  pub dialog_elements: BTreeMap<String, usize>,
  pub phrase_elements: BTreeMap<String, usize>,
}

impl DialogSweepCensus {
  /// Counts one more `key` in `map`.
  pub fn count(map: &mut BTreeMap<String, usize>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
  }
}
