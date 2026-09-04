use std::sync::Arc;

use xrf_error::XrfResult;

use crate::document::ltx_document::LtxDocument;

/// What resolving a config tree asks of the world holding it.
pub trait LtxDocumentSource {
  /// One config as a parsed document, or `None` when nothing holds it.
  ///
  /// Absence is not an error: a wildcard match that vanished, or a config a generator has not produced, is nothing to
  /// merge. A caller that needs a named include to exist says so itself.
  fn read_document(&self, logical_path: &str) -> XrfResult<Option<Arc<LtxDocument>>>;

  /// Files an `#include` statement names, resolved against `directory`, sorted.
  ///
  /// A statement carrying `*` expands to every match; one without expands to itself whether or not it exists. Sorted
  /// so merge order does not depend on how a directory happened to enumerate.
  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>>;

  /// Every file name directly in `directory`, names only and no deeper.
  ///
  /// What lets a dialect apply its own naming rules to a config's siblings, which is how DLTX finds the files that
  /// patch a root. A dialect with no such rule never calls it.
  fn list_file_names(&self, directory: &str) -> XrfResult<Vec<String>>;
}
