use crate::document::{LtxKeyOperation, LtxSectionOperation};

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
