use std::ops::Range;

use crate::element::{DialogElement, DialogElementKind};

/// One line of a conversation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogPhrase {
  id: String,
  elements: Vec<DialogElement>,
  range: Range<usize>,
  is_in_phrase_list: bool,
}

impl DialogPhrase {
  pub fn new(id: String, elements: Vec<DialogElement>, range: Range<usize>, is_in_phrase_list: bool) -> Self {
    Self {
      id,
      elements,
      range,
      is_in_phrase_list,
    }
  }

  /// The phrase id, unique within its dialog and referenced by `next`.
  ///
  /// The engine cares only that a dialog's entry phrase is `0`; the rest are opaque labels, so hand
  /// written dialogs use descriptive ones.
  pub fn get_id(&self) -> &str {
    &self.id
  }

  /// Child elements in document order.
  pub fn get_elements(&self) -> &[DialogElement] {
    &self.elements
  }

  /// Byte range of the whole `phrase` element in the document source.
  pub fn get_range(&self) -> &Range<usize> {
    &self.range
  }

  /// Whether the phrase sits inside a `phrase_list` rather than directly under its dialog.
  ///
  /// Both forms occur, and an insertion has to reproduce the one the file already uses.
  pub fn is_in_phrase_list(&self) -> bool {
    self.is_in_phrase_list
  }

  /// Values of every element of one kind, in document order.
  pub fn list_values_of(&self, kind: DialogElementKind) -> Vec<&str> {
    self
      .elements
      .iter()
      .filter(|element| element.get_kind() == kind)
      .map(DialogElement::get_value)
      .collect()
  }

  /// Ids of the phrases that may follow this one, **in the order the player sees them**.
  ///
  /// File order is presentation order, so this sequence is game behavior rather than a detail of the
  /// file. Reordering it changes what the player is offered first.
  pub fn list_next(&self) -> Vec<&str> {
    self.list_values_of(DialogElementKind::Next)
  }

  /// Translation key of the line, when it has one.
  ///
  /// An empty `<text></text>` answers `None`, because an empty string is not a key: nothing in a
  /// string table is named by it, and treating it as one reports the phrase as pointing at a missing
  /// translation. Anomaly writes 2,527 of them — phrases carrying only actions, or a silent option —
  /// so the difference decides whether validation is usable on a real project or drowns in false
  /// alarms. The literal element is still reachable through [`Self::get_elements`].
  ///
  /// A phrase carries at most one `text` in shipped data, but the parser does not enforce that; a
  /// second one is reachable through `get_elements` too.
  pub fn get_text(&self) -> Option<&str> {
    self
      .elements
      .iter()
      .find(|element| element.get_kind() == DialogElementKind::Text)
      .map(DialogElement::get_value)
      .filter(|value| !value.is_empty())
  }

  /// Whether selecting this phrase ends the conversation.
  pub fn is_final(&self) -> bool {
    self
      .elements
      .iter()
      .any(|element| element.get_kind() == DialogElementKind::IsFinal && element.is_enabled())
  }

  pub fn has_element_of(&self, kind: DialogElementKind) -> bool {
    self.elements.iter().any(|element| element.get_kind() == kind)
  }
}
