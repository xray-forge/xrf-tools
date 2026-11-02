use std::ops::Range;

use xrf_xml::XmlElementSpan;

use crate::conversation::dialog_element_kind::DialogElementKind;

/// One child element of a dialog or a phrase, as written.
///
/// Elements are held in an ordered list rather than as named fields because nearly every one of them
/// repeats, and because `next` order decides the order the player sees the options in. A struct of
/// `Option` fields would lose both facts, and a set would lose the second silently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogElement {
  name: String,
  value: String,
  range: Range<usize>,
}

impl DialogElement {
  pub fn new(name: String, value: String, range: Range<usize>) -> Self {
    Self { name, value, range }
  }

  /// The element name as written, which is what a rewrite has to reproduce.
  pub fn get_name(&self) -> &str {
    &self.name
  }

  /// Text content with entity references resolved.
  pub fn get_value(&self) -> &str {
    &self.value
  }

  /// Byte range of the whole element in the document source.
  pub fn get_range(&self) -> &Range<usize> {
    &self.range
  }

  pub fn get_kind(&self) -> DialogElementKind {
    DialogElementKind::from_name(&self.name)
  }

  /// Whether the value reads as set.
  ///
  /// The engine treats any content as true for a flag element; shipped data writes `1`.
  pub fn is_enabled(&self) -> bool {
    !self.value.trim().is_empty() && self.value.trim() != "0"
  }
}

/// The element as written: its name, its text, and where it sits.
impl From<&XmlElementSpan> for DialogElement {
  fn from(element: &XmlElementSpan) -> Self {
    Self::new(
      element.name().to_owned(),
      element.text().to_owned(),
      element.element_range().clone(),
    )
  }
}
