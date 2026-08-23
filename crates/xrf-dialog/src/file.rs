use std::path::Path;

use xrf_error::XrfResult;
use xrf_utils::XRayEncoding;
use xrf_xml::{XmlElementSpan, XmlParseOptions, XmlSourceDocument};

use crate::constants::{
  DIALOG_ATTRIBUTES, DIALOG_ELEMENT, ID_ATTRIBUTE, PHRASE_ATTRIBUTES, PHRASE_ELEMENT, PHRASE_LIST_ELEMENT,
  PRIORITY_ATTRIBUTE, ROOT_ELEMENT,
};
use crate::dialog::Dialog;
use crate::element::{DialogElement, DialogElementKind};
use crate::encoding::{DecodedDialogSource, decode, read_decoded};
use crate::issue::{DialogParseIssue, DialogParseIssueKind};
use crate::phrase::DialogPhrase;

/// One dialog XML document, with the text every range in it addresses.
///
/// The source travels with the model on purpose. Every range a dialog or phrase reports is a byte
/// offset into this exact string, so an edit splices what was actually on disk, keeping the comment
/// banners and indentation that re-serializing would drop.
#[derive(Debug)]
pub struct DialogFile {
  source: String,
  dialogs: Vec<Dialog>,
  issues: Vec<DialogParseIssue>,
  encoding: XRayEncoding,
  byte_order_mark: Vec<u8>,
}

impl DialogFile {
  /// Read and parse a dialog file from disk.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the file cannot be read, an encoding error when its bytes do not
  /// decode, and a parsing error when the document is malformed beyond what the engine accepts.
  pub fn read_from_path(path: &Path) -> XrfResult<Self> {
    Self::from_decoded(read_decoded(path)?)
  }

  /// Parse dialog bytes already in hand, such as an entry read out of an archive.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when the bytes do not decode, and a parsing error when the document is
  /// malformed beyond what the engine accepts.
  pub fn read_from_bytes(data: &[u8]) -> XrfResult<Self> {
    Self::from_decoded(decode(data)?)
  }

  /// Parse decoded text, taking the encoding it was decoded with on trust.
  ///
  /// For a caller that already holds the text and does not intend to write it back.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when the document is malformed beyond what the engine accepts.
  pub fn parse(source: String, encoding: XRayEncoding) -> XrfResult<Self> {
    Self::from_decoded(DecodedDialogSource {
      byte_order_mark: Vec::new(),
      encoding,
      text: source,
    })
  }

  /// The text every range in this document addresses.
  pub fn get_source(&self) -> &str {
    &self.source
  }

  /// Dialogs in document order.
  pub fn get_dialogs(&self) -> &[Dialog] {
    &self.dialogs
  }

  /// Everything the parser read but the schema does not describe.
  pub fn get_issues(&self) -> &[DialogParseIssue] {
    &self.issues
  }

  /// The encoding the source was decoded with, and the one a rewrite has to use.
  pub fn get_encoding(&self) -> XRayEncoding {
    self.encoding
  }

  /// The leading byte order mark, kept verbatim so a rewrite does not drop it.
  pub fn get_byte_order_mark(&self) -> &[u8] {
    &self.byte_order_mark
  }

  /// The first dialog with this id.
  ///
  /// Duplicate ids across files occur and are a validation concern, not a parsing one.
  pub fn find_dialog(&self, id: &str) -> Option<&Dialog> {
    self.dialogs.iter().find(|dialog| dialog.get_id() == id)
  }

  /// Total phrases across every dialog.
  pub fn sum_phrases(&self) -> usize {
    self.dialogs.iter().map(|dialog| dialog.get_phrases().len()).sum()
  }

  fn from_decoded(decoded: DecodedDialogSource) -> XrfResult<Self> {
    let document: XmlSourceDocument = XmlSourceDocument::parse(decoded.text, XmlParseOptions::default())?;
    let mut issues: Vec<DialogParseIssue> = Vec::new();
    let mut dialogs: Vec<Dialog> = Vec::new();

    let root: &XmlElementSpan = document.root();

    // The engine reads `game_dialogs`, but a differently named root is worth reporting rather than
    // refusing: a file assembled by a tool that got it wrong still holds readable dialogs.
    if root.name() != ROOT_ELEMENT {
      issues.push(DialogParseIssue::new(
        DialogParseIssueKind::UnknownElement,
        None,
        None,
        root.name().to_owned(),
        root.element_range().clone(),
      ));
    }

    for element in root.children_named(DIALOG_ELEMENT) {
      if let Some(dialog) = read_dialog(element, &mut issues) {
        dialogs.push(dialog);
      }
    }

    Ok(Self {
      source: document.into_source(),
      dialogs,
      issues,
      encoding: decoded.encoding,
      byte_order_mark: decoded.byte_order_mark,
    })
  }
}

/// Read one `dialog`, collecting what the schema does not describe.
///
/// Answers `None` only for a dialog with no id, which nothing can address.
fn read_dialog(element: &XmlElementSpan, issues: &mut Vec<DialogParseIssue>) -> Option<Dialog> {
  let id: &str = match element.attribute(ID_ATTRIBUTE) {
    Some(id) => id,
    None => {
      issues.push(DialogParseIssue::new(
        DialogParseIssueKind::MissingId,
        None,
        None,
        String::from(DIALOG_ELEMENT),
        element.element_range().clone(),
      ));

      return None;
    }
  };

  report_unknown_attributes(element, DIALOG_ATTRIBUTES, Some(id), None, issues);

  let priority: Option<i32> = read_priority(element, id, issues);
  let mut elements: Vec<DialogElement> = Vec::new();
  let mut phrases: Vec<DialogPhrase> = Vec::new();

  for child in element.children() {
    match child.name() {
      // Both shapes occur: 499 dialogs wrap their phrases and one writes a bare `<phrase id="0"/>`
      // straight under the dialog. Each phrase remembers which, so an insertion matches the file.
      PHRASE_LIST_ELEMENT => {
        for phrase in child.children_named(PHRASE_ELEMENT) {
          if let Some(phrase) = read_phrase(phrase, id, true, issues) {
            phrases.push(phrase);
          }
        }
      }
      PHRASE_ELEMENT => {
        if let Some(phrase) = read_phrase(child, id, false, issues) {
          phrases.push(phrase);
        }
      }
      name => {
        let kind: DialogElementKind = DialogElementKind::from_name(name);

        if !kind.is_valid_for_dialog() {
          issues.push(DialogParseIssue::new(
            DialogParseIssueKind::UnknownElement,
            Some(id.to_owned()),
            None,
            name.to_owned(),
            child.element_range().clone(),
          ));
        }

        elements.push(new_element(child));
      }
    }
  }

  Some(Dialog::new(
    id.to_owned(),
    priority,
    elements,
    phrases,
    element.element_range().clone(),
  ))
}

/// Read one `phrase`, collecting what the schema does not describe.
///
/// Answers `None` only for a phrase with no id, which no `next` can reach.
fn read_phrase(
  element: &XmlElementSpan,
  dialog_id: &str,
  is_in_phrase_list: bool,
  issues: &mut Vec<DialogParseIssue>,
) -> Option<DialogPhrase> {
  let id: &str = match element.attribute(ID_ATTRIBUTE) {
    Some(id) => id,
    None => {
      issues.push(DialogParseIssue::new(
        DialogParseIssueKind::MissingId,
        Some(dialog_id.to_owned()),
        None,
        String::from(PHRASE_ELEMENT),
        element.element_range().clone(),
      ));

      return None;
    }
  };

  report_unknown_attributes(element, PHRASE_ATTRIBUTES, Some(dialog_id), Some(id), issues);

  let mut elements: Vec<DialogElement> = Vec::new();

  for child in element.children() {
    if !DialogElementKind::from_name(child.name()).is_valid_for_phrase() {
      issues.push(DialogParseIssue::new(
        DialogParseIssueKind::UnknownElement,
        Some(dialog_id.to_owned()),
        Some(id.to_owned()),
        child.name().to_owned(),
        child.element_range().clone(),
      ));
    }

    elements.push(new_element(child));
  }

  Some(DialogPhrase::new(
    id.to_owned(),
    elements,
    element.element_range().clone(),
    is_in_phrase_list,
  ))
}

/// Read `priority`, reporting a value that is not an integer rather than failing the file.
fn read_priority(element: &XmlElementSpan, id: &str, issues: &mut Vec<DialogParseIssue>) -> Option<i32> {
  let raw: &str = element.attribute(PRIORITY_ATTRIBUTE)?;

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

fn report_unknown_attributes(
  element: &XmlElementSpan,
  known: &[&str],
  dialog_id: Option<&str>,
  phrase_id: Option<&str>,
  issues: &mut Vec<DialogParseIssue>,
) {
  for (name, _) in element.attributes() {
    if !known.contains(&name) {
      issues.push(DialogParseIssue::new(
        DialogParseIssueKind::UnknownAttribute,
        dialog_id.map(str::to_owned),
        phrase_id.map(str::to_owned),
        name.to_owned(),
        element.element_range().clone(),
      ));
    }
  }
}

fn new_element(element: &XmlElementSpan) -> DialogElement {
  DialogElement::new(
    element.name().to_owned(),
    element.text().to_owned(),
    element.element_range().clone(),
  )
}
