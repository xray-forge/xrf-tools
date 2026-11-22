use std::collections::HashSet;

use xrf_dialog::{Dialog, DialogElementKind, DialogPhrase};

/// One dialog's phrases as `CPhraseDialog::load_shared` builds its graph from them.
///
/// The engine reads only the phrases inside `phrase_list`, finds each by its id with the first match winning, and walks
/// from phrase `0` through every `next`, adding each id once. A dialog without a phrase list is built by its `init_func`
/// instead.
pub(crate) struct DialogPhraseGraph<'a> {
  dialog: &'a Dialog,
  listed: Vec<&'a DialogPhrase>,
}

impl<'a> DialogPhraseGraph<'a> {
  pub(crate) fn new(dialog: &'a Dialog) -> Self {
    Self {
      dialog,
      listed: dialog
        .get_phrases()
        .iter()
        .filter(|phrase| phrase.is_in_phrase_list())
        .collect(),
    }
  }

  /// Whether the dialog lists no phrases, so the engine hands it to its `init_func` to build.
  pub(crate) fn is_scripted(&self) -> bool {
    self.listed.is_empty()
  }

  /// Whether a scripted dialog names the `init_func` the engine calls for it.
  pub(crate) fn has_init_func(&self) -> bool {
    !self.dialog.list_values_of(DialogElementKind::InitFunc).is_empty()
  }

  /// The listed phrase the engine finds for an id: the first one carrying it.
  pub(crate) fn find(&self, id: &str) -> Option<&'a DialogPhrase> {
    self.listed.iter().copied().find(|phrase| phrase.get_id() == id)
  }

  /// Every phrase the engine loads, in the order it reaches them from phrase `0`; none without phrase `0`.
  pub(crate) fn list_reached(&self) -> Vec<&'a DialogPhrase> {
    let mut reached: Vec<&'a DialogPhrase> = Vec::new();
    let mut seen: HashSet<&'a str> = HashSet::new();
    let mut pending: Vec<&'a DialogPhrase> = self.find(Dialog::ENTRY_PHRASE_ID).into_iter().collect();

    // Depth first, as the engine's recursion visits them; an id already added is not walked again, which ends cycles.
    while let Some(phrase) = pending.pop() {
      if !seen.insert(phrase.get_id()) {
        continue;
      }

      reached.push(phrase);
      pending.extend(phrase.list_next().into_iter().rev().filter_map(|next| self.find(next)));
    }

    reached
  }

  /// Ids of the listed phrases no path from phrase `0` reaches, each once, in list order.
  pub(crate) fn list_unreached_ids(&self, reached: &[&'a DialogPhrase]) -> Vec<&'a str> {
    let mut seen: HashSet<&'a str> = reached.iter().map(|phrase| phrase.get_id()).collect();

    self
      .listed
      .iter()
      .map(|phrase| phrase.get_id())
      .filter(|id| seen.insert(id))
      .collect()
  }
}
