use crate::document::{LtxKeyOperation, LtxSectionOperation, LtxSpan};

/// One statement of an LTX file, with where it was written.
///
/// Deliberately small: a project retains one of these per statement of every config it holds, so a field that only
/// some callers want does not live here. The authored line is the one such field, and it sits on
/// [`crate::document::LtxDocument`] instead.
#[derive(Clone, Debug, PartialEq)]
pub struct LtxItem {
  pub kind: LtxItemKind,
  pub span: LtxSpan,
}

/// What a statement says.
///
/// There is no blank-line kind. Every statement occupies exactly one line, so a gap between consecutive spans says how
/// many lines held nothing, and a whitespace-only line is normalized to empty - which is what the formatter already
/// does with it.
///
/// Text is `Box<str>` rather than `String`: a statement is parsed once and never edited, so the growth capacity a
/// `String` carries is eight bytes per field that nothing can ever use.
#[derive(Clone, Debug, PartialEq)]
pub enum LtxItemKind {
  /// A whole-line comment, without its leading `;`.
  Comment { text: Box<str> },
  /// An `#include` naming one file or one wildcard mask.
  Include { path: Box<str>, comment: Option<Box<str>> },
  /// A section header, with the parents it declares and the operation its prefix asks for.
  Section {
    name: Box<str>,
    operation: LtxSectionOperation,
    parents: Box<[Box<str>]>,
    comment: Option<Box<str>>,
  },
  /// A field line. `value` is `None` for a bare key, which is not the same as an empty value.
  Key {
    name: Box<str>,
    value: Option<Box<str>>,
    operation: LtxKeyOperation,
    comment: Option<Box<str>>,
  },
}

impl LtxItem {
  pub(crate) fn new(kind: LtxItemKind, span: LtxSpan) -> Self {
    Self { kind, span }
  }

  /// The path and comment of an `#include`, or `None` for any other statement.
  pub fn as_include(&self) -> Option<(&str, Option<&str>)> {
    match &self.kind {
      LtxItemKind::Include { path, comment } => Some((path, comment.as_deref())),
      _ => None,
    }
  }

  /// Whether this statement carries a DLTX operation prefix.
  ///
  /// Standard resolution refuses these rather than treating a prefix as part of a name; no shipped config uses one as
  /// an ordinary key, so the reading is unambiguous.
  pub fn is_patch_operation(&self) -> bool {
    match &self.kind {
      LtxItemKind::Section { operation, .. } => operation.is_patch(),
      LtxItemKind::Key { operation, .. } => operation.is_patch(),
      _ => false,
    }
  }
}
