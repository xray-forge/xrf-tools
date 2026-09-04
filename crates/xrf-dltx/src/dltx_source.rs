use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_ltx::LtxDocument;

/// What DLTX evaluation asks of the world holding the configs.
pub trait DltxSource {
  /// One config as a parsed document, or `None` when nothing holds it.
  ///
  /// Absence is not an error here: the engine treats a missing wildcard match as nothing to merge, and a caller
  /// decides whether a named include that is missing is worth reporting.
  fn read_document(&self, logical_path: &str) -> XrfResult<Option<Arc<LtxDocument>>>;

  /// Files an `#include` statement names, resolved against `directory`, sorted.
  ///
  /// A statement carrying `*` expands to every match; one without expands to itself whether or not it exists.
  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>>;

  /// Every file name directly in `directory`, so mod-file discovery can apply the engine's naming rules to it.
  ///
  /// Names only, not paths, and only that one directory: the engine searches beside the root file and no deeper.
  fn list_file_names(&self, directory: &str) -> XrfResult<Vec<String>>;
}
