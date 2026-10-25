use std::ops::Range;

use xrf_xml::XmlElementSpan;

use crate::conversation::dialog_element::DialogElement;
use crate::conversation::dialog_element_kind::DialogElementKind;
use crate::conversation::dialog_parse_issue::DialogParseIssue;
use crate::conversation::dialog_parse_issue_kind::DialogParseIssueKind;
use crate::conversation::dialog_phrase::DialogPhrase;

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
  /// The phrase a dialog opens with, said by whoever opens it.
  pub const ENTRY_PHRASE_ID: &'static str = "0";

  /// The element a dialog file declares each dialog with.
  pub(crate) const ELEMENT: &'static str = "dialog";
  const ID_ATTRIBUTE: &'static str = "id";
  const PRIORITY_ATTRIBUTE: &'static str = "priority";
  /// Attributes a `dialog` may carry.
  const ATTRIBUTES: &'static [&'static str] = &[Self::ID_ATTRIBUTE, Self::PRIORITY_ATTRIBUTE];

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

  /// The phrase the dialog opens with, whose line the topic list shows for the dialog.
  pub fn get_entry_phrase(&self) -> Option<&DialogPhrase> {
    self.find_phrase(Self::ENTRY_PHRASE_ID)
  }

  /// Whether a phrase leads back to the entry phrase, the only way the engine ever reads its own conditions.
  pub fn is_entry_phrase_revisited(&self) -> bool {
    self
      .phrases
      .iter()
      .any(|phrase| phrase.list_next().contains(&Self::ENTRY_PHRASE_ID))
  }

  /// Whether the dialog declares phrases at all.
  ///
  /// A dialog with none is not malformed: `dm_traveler_dialog` carries only a precondition and an
  /// init function, and builds its phrases from script at runtime.
  pub fn has_phrases(&self) -> bool {
    !self.phrases.is_empty()
  }

  /// Read one `dialog`, collecting what the schema does not describe.
  ///
  /// Answers `None` only for a dialog with no id, which nothing can address.
  pub(crate) fn read(element: &XmlElementSpan, issues: &mut Vec<DialogParseIssue>) -> Option<Self> {
    let Some(id) = element.attribute(Self::ID_ATTRIBUTE) else {
      issues.push(DialogParseIssue::new(
        DialogParseIssueKind::MissingId,
        None,
        None,
        String::from(Self::ELEMENT),
        element.element_range().clone(),
      ));

      return None;
    };

    DialogParseIssue::report_unknown_attributes(element, Self::ATTRIBUTES, Some(id), None, issues);

    let priority: Option<i32> = Self::read_priority(element, id, issues);
    let mut elements: Vec<DialogElement> = Vec::new();
    let mut phrases: Vec<DialogPhrase> = Vec::new();

    for child in element.children() {
      match child.name() {
        // Both shapes occur: 499 dialogs wrap their phrases and one writes a bare `<phrase id="0"/>`
        // straight under the dialog. Each phrase remembers which, so an insertion matches the file.
        DialogPhrase::LIST_ELEMENT => phrases.extend(
          child
            .children_named(DialogPhrase::ELEMENT)
            .filter_map(|phrase| DialogPhrase::read(phrase, id, true, issues)),
        ),
        DialogPhrase::ELEMENT => phrases.extend(DialogPhrase::read(child, id, false, issues)),
        name => {
          if !DialogElementKind::from_name(name).is_valid_for_dialog() {
            issues.push(DialogParseIssue::new(
              DialogParseIssueKind::UnknownElement,
              Some(id.to_owned()),
              None,
              name.to_owned(),
              child.element_range().clone(),
            ));
          }

          elements.push(DialogElement::from(child));
        }
      }
    }

    Some(Self::new(
      id.to_owned(),
      priority,
      elements,
      phrases,
      element.element_range().clone(),
    ))
  }

  /// Read `priority`, reporting a value that is not an integer rather than failing the file.
  fn read_priority(element: &XmlElementSpan, id: &str, issues: &mut Vec<DialogParseIssue>) -> Option<i32> {
    let raw: &str = element.attribute(Self::PRIORITY_ATTRIBUTE)?;

    match raw.trim().parse::<i32>() {
      Ok(priority) => Some(priority),
      Err(_) => {
        issues.push(DialogParseIssue::new(
          DialogParseIssueKind::InvalidPriority,
          Some(id.to_owned()),
          None,
          raw.to_owned(),
          element.element_range().clone(),
        ));

        None
      }
    }
  }
}
