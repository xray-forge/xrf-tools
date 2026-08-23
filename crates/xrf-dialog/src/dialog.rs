use std::ops::Range;

use crate::element::{DialogElement, DialogElementKind};
use crate::phrase::DialogPhrase;

/// One conversation: its identity, the conditions gating it, and its phrases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dialog {
  id: String,
  priority: Option<i32>,
  elements: Vec<DialogElement>,
  phrases: Vec<DialogPhrase>,
  range: Range<usize>,
}

impl Dialog {
  pub fn new(
    id: String,
    priority: Option<i32>,
    elements: Vec<DialogElement>,
    phrases: Vec<DialogPhrase>,
    range: Range<usize>,
  ) -> Self {
    Self {
      id,
      priority,
      elements,
      phrases,
      range,
    }
  }

  pub fn get_id(&self) -> &str {
    &self.id
  }

  /// Selection priority, which is negative for a dialog meant to sort last.
  pub fn get_priority(&self) -> Option<i32> {
    self.priority
  }

  /// Dialog-level child elements in document order, excluding the phrases.
  pub fn get_elements(&self) -> &[DialogElement] {
    &self.elements
  }

  /// Phrases in document order, whether they were written inside a `phrase_list` or not.
  pub fn get_phrases(&self) -> &[DialogPhrase] {
    &self.phrases
  }

  /// Byte range of the whole `dialog` element in the document source.
  pub fn get_range(&self) -> &Range<usize> {
    &self.range
  }

  /// Values of every dialog-level element of one kind, in document order.
  pub fn list_values_of(&self, kind: DialogElementKind) -> Vec<&str> {
    self
      .elements
      .iter()
      .filter(|element| element.get_kind() == kind)
      .map(DialogElement::get_value)
      .collect()
  }

  /// The first phrase carrying an id, or `None` for a dialog that declares no phrases.
  ///
  /// Matched by id rather than by position: duplicates occur, and the first match is what the engine
  /// reaches too.
  pub fn find_phrase(&self, id: &str) -> Option<&DialogPhrase> {
    self.phrases.iter().find(|phrase| phrase.get_id() == id)
  }

  /// Whether the dialog declares phrases at all.
  ///
  /// A dialog with none is not malformed: `dm_traveler_dialog` carries only a precondition and an
  /// init function, and builds its phrases from script at runtime.
  pub fn has_phrases(&self) -> bool {
    !self.phrases.is_empty()
  }
}
