use crate::document::ltx_key_operation::LtxKeyOperation;
use crate::document::ltx_section_operation::LtxSectionOperation;
use crate::document::ltx_span::LtxSpan;

/// One statement of an LTX file, with where it was written.
#[derive(Clone, Debug, PartialEq)]
pub struct LtxItem {
  pub kind: LtxItemKind,
  pub span: LtxSpan,
  /// The line exactly as authored, retained only under `preserve_source`.
  ///
  /// What lets an editor write a file back with untouched lines byte-identical. Verification and formatting never read
  /// it, so they do not pay for it.
  pub source: Option<String>,
}

/// What a statement says.
///
/// There is no blank-line kind. Every statement occupies exactly one line, so a gap between consecutive spans says how
/// many lines held nothing, and a whitespace-only line is normalized to empty - which is what the formatter already
/// does with it.
#[derive(Clone, Debug, PartialEq)]
pub enum LtxItemKind {
  /// A whole-line comment, without its leading `;`.
  Comment { text: String },
  /// An `#include` naming one file or one wildcard mask.
  Include { path: String, comment: Option<String> },
  /// A section header, with the parents it declares and the operation its prefix asks for.
  Section {
    name: String,
    operation: LtxSectionOperation,
    parents: Vec<String>,
    comment: Option<String>,
  },
  /// A field line. `value` is `None` for a bare key, which is not the same as an empty value.
  Key {
    name: String,
    value: Option<String>,
    operation: LtxKeyOperation,
    comment: Option<String>,
  },
}

impl LtxItem {
  pub(crate) fn new(kind: LtxItemKind, span: LtxSpan) -> Self {
    Self {
      kind,
      source: None,
      span,
    }
  }

  pub(crate) fn with_source(mut self, source: Option<String>) -> Self {
    self.source = source;

    self
  }

  /// The path and comment of an `#include`, or `None` for any other statement.
  pub fn as_include(&self) -> Option<(&str, Option<&str>)> {
    match &self.kind {
      LtxItemKind::Include { path, comment } => Some((path.as_str(), comment.as_deref())),
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
