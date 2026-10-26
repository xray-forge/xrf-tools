use crate::document::{LtxItemKind, LtxSpan};

/// One statement of an LTX file, with where it was written.
///
/// Statements are read in order off a [`crate::LtxDocument`], which also says what a statement cannot: which section a
/// key belongs to is the nearest header above it.
#[derive(Clone, Debug, PartialEq)]
pub struct LtxItem {
  pub kind: LtxItemKind,
  pub span: LtxSpan,
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
