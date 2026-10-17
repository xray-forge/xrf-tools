use serde::Serialize;

use crate::commands::profile::run::profiled_binary_output::ProfiledBinaryOutput;

/// What one profiling session measured.
///
/// Carries the protocol alongside the numbers — rounds, warmup, the command, and each binary's own account of itself —
/// because a duration without them cannot be compared to anything. This is the whole reason the harness exists rather
/// than a stopwatch and a note.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileReportOutput {
  /// The argument vector handed to every binary, so a reader never reconstructs it from prose.
  pub command: Vec<String>,
  pub rounds: usize,
  /// Rounds run and discarded before measuring, because a cold file cache costs up to twice the warm figure.
  pub warmup: usize,
  pub binaries: Vec<ProfiledBinaryOutput>,
}
