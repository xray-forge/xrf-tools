use std::path::Path;

use xrf_error::XrfResult;
use xrf_utils::XRayEncoding;
use xrf_xml::{XmlElementSpan, XmlParseOptions, XmlSourceDocument};

use crate::conversation::dialog::Dialog;
use crate::conversation::dialog_parse_issue::DialogParseIssue;
use crate::conversation::dialog_parse_issue_kind::DialogParseIssueKind;
use crate::dialog_source::DialogSource;

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
  /// The root element the engine reads dialogs from.
  const ROOT_ELEMENT: &'static str = "game_dialogs";

  /// Read and parse a dialog file from disk.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the file cannot be read, an encoding error when its bytes do not
  /// decode, and a parsing error when the document is malformed beyond what the engine accepts.
  pub fn read_from_path(path: &Path) -> XrfResult<Self> {
    Self::from_source(DialogSource::read(path)?)
  }

  /// Parse dialog bytes already in hand, such as an entry read out of an archive.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when the bytes do not decode, and a parsing error when the document is
  /// malformed beyond what the engine accepts.
  pub fn read_from_bytes(data: &[u8]) -> XrfResult<Self> {
    Self::from_source(DialogSource::decode(data)?)
  }

  /// Parse decoded text, taking the encoding it was decoded with on trust.
  ///
  /// For a caller that already holds the text and does not intend to write it back.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when the document is malformed beyond what the engine accepts.
  pub fn parse(source: String, encoding: XRayEncoding) -> XrfResult<Self> {
    Self::from_source(DialogSource {
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

  fn from_source(source: DialogSource) -> XrfResult<Self> {
    let document: XmlSourceDocument = XmlSourceDocument::parse(source.text, XmlParseOptions::default())?;
    let mut issues: Vec<DialogParseIssue> = Vec::new();

    let root: &XmlElementSpan = document.root();

    // The engine reads `game_dialogs`, but a differently named root is worth reporting rather than
    // refusing: a file assembled by a tool that got it wrong still holds readable dialogs.
    if root.name() != Self::ROOT_ELEMENT {
      issues.push(DialogParseIssue::new(
        DialogParseIssueKind::UnknownElement,
        None,
        None,
        root.name().to_owned(),
        root.element_range().clone(),
      ));
    }

    let dialogs: Vec<Dialog> = root
      .children_named(Dialog::ELEMENT)
      .filter_map(|element| Dialog::read(element, &mut issues))
      .collect();

    Ok(Self {
      source: document.into_source(),
      dialogs,
      issues,
      encoding: source.encoding,
      byte_order_mark: source.byte_order_mark,
    })
  }
}
