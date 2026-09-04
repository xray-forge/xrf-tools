use std::path::{Path, PathBuf};

use xrf_error::XrfResult;

use crate::ltx::Ltx;

/// Where an `#include` statement is resolved and read from.
///
/// Exists so the merge rules in [`crate::ltx::LtxIncludeConvertor`] are written once. Those rules - a wildcard
/// expanding to several files, a duplicate section being an error unless it is the root one - are the same whether the
/// includes sit on disk or inside archive volumes; only finding and reading them differs.
///
/// Paths are [`PathBuf`] for both backends so [`Ltx::path`] and [`Ltx::directory`] keep their type. For a VFS-backed source
/// they carry X-Ray logical paths rather than filesystem ones, which round-trip through `PathBuf` unharmed and are only ever
/// handed back to the same source.
pub(crate) trait LtxIncludeSource {
  /// Paths an include statement names, resolved against the including file's directory.
  ///
  /// A statement carrying `*` expands to every match, sorted, so section merging is deterministic.
  fn resolve(&self, directory: &Path, statement: &str) -> XrfResult<Vec<PathBuf>>;

  /// Reads one included file.
  ///
  /// Answers `None` for an include that is absent or is a variant this reader skips, which the convertor treats as nothing
  /// to merge rather than as a failure.
  fn read(&self, path: &Path) -> XrfResult<Option<Ltx>>;

  /// How a path reads in an error message.
  ///
  /// Separate from `Display` because a logical path is not a filesystem path, and printing it as one invites someone to go
  /// looking for a file that does not exist.
  fn describe(&self, path: &Path) -> String;
}
