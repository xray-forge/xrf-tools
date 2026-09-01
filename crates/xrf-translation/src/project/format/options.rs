use std::path::PathBuf;

use xrf_utils::LineEndings;

/// What one formatting run was asked to do.
pub struct ProjectFormatOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub output: xrf_output::OutputOptions,
  /// Files and directories to normalize, as host paths.
  ///
  /// Host paths rather than VFS roots, because this rewrites its sources in place and there is nowhere to put a file
  /// inside an archive volume. `initialize` is a host walk for the same reason.
  pub paths: Vec<PathBuf>,
  /// Report what would change without writing any of it.
  pub is_check: bool,
  /// Write these line endings rather than preserving each file's own.
  ///
  /// Also arms the check: with a choice made, a file spelling its endings the other way is not formatted, and the check
  /// says so. Without one, the comparison normalizes both sides and endings are left to `.gitattributes`.
  pub line_endings: Option<LineEndings>,
}
