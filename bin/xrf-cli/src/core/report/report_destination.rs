use std::path::PathBuf;

/// Where a run's machine-readable account of itself goes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportDestination {
  /// Nothing machine-readable was asked for, so nothing is produced.
  None,
  /// One JSON document on stdout, for a caller that pipes it.
  Stdout,
  /// One JSON document at the caller's path, written after the run.
  File(PathBuf),
}

impl ReportDestination {
  /// Whether a command's structured result is going to be read by anyone.
  pub const fn is_requested(&self) -> bool {
    !matches!(self, Self::None)
  }

  /// Whether stdout is still free for human-facing results.
  pub const fn is_stdout_available(&self) -> bool {
    !matches!(self, Self::Stdout)
  }
}
