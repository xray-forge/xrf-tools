use crate::document::{LtxKeyOperation, LtxSectionOperation};

/// What a statement says.
///
/// There is no blank-line kind. Every statement occupies exactly one line, so a gap between consecutive spans says how
/// many lines held nothing, and a whitespace-only line is normalized to empty - which is what the formatter already
/// does with it.
///
/// Text is `Box<str>` rather than `String` in a held document: a statement is parsed once and never edited, so the
/// growth capacity a `String` carries is eight bytes per field that nothing can ever use. The parser reads each
/// statement with its text borrowed from the input (`S = &str`), and only a document it is kept in owns a copy: a
/// statement lowered as it is read never allocates its text twice.
#[derive(Clone, Debug, PartialEq)]
pub enum LtxItemKind<S = Box<str>> {
  /// A whole-line comment, without its leading `;`.
  Comment { text: S },
  /// An `#include` naming one file or one wildcard mask.
  Include { path: S, comment: Option<S> },
  /// A section header, with the parents it declares and the operation its prefix asks for.
  Section {
    name: S,
    operation: LtxSectionOperation,
    parents: Box<[S]>,
    comment: Option<S>,
  },
  /// A field line. `value` is `None` for a bare key, which is not the same as an empty value.
  Key {
    name: S,
    value: Option<S>,
    operation: LtxKeyOperation,
    comment: Option<S>,
  },
}

impl From<LtxItemKind<&str>> for LtxItemKind {
  fn from(kind: LtxItemKind<&str>) -> Self {
    match kind {
      LtxItemKind::Comment { text } => Self::Comment { text: Box::from(text) },
      LtxItemKind::Include { path, comment } => Self::Include {
        path: Box::from(path),
        comment: comment.map(Box::from),
      },
      LtxItemKind::Section {
        name,
        operation,
        parents,
        comment,
      } => Self::Section {
        name: Box::from(name),
        operation,
        parents: parents.iter().copied().map(Box::from).collect(),
        comment: comment.map(Box::from),
      },
      LtxItemKind::Key {
        name,
        value,
        operation,
        comment,
      } => Self::Key {
        name: Box::from(name),
        value: value.map(Box::from),
        operation,
        comment: comment.map(Box::from),
      },
    }
  }
}
