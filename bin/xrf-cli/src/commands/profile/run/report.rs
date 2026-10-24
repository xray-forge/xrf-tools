use serde::Serialize;

use crate::commands::profile::run::profile_run_binary_report::ProfileRunBinaryReport;

/// What one profiling session measured.
///
/// Carries the protocol alongside the numbers — rounds, warmup, the command, and each binary's own account of itself —
/// because a duration without them cannot be compared to anything. This is the whole reason the harness exists rather
/// than a stopwatch and a note.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRunReport {
  /// The argument vector handed to every binary, so a reader never reconstructs it from prose.
  command: Vec<String>,
  rounds: usize,
  /// Rounds run and discarded before measuring, because a cold file cache costs up to twice the warm figure.
  warmup: usize,
  binaries: Vec<ProfileRunBinaryReport>,
}

impl ProfileRunReport {
  pub fn new(command: Vec<String>, rounds: usize, warmup: usize, binaries: Vec<ProfileRunBinaryReport>) -> Self {
    Self {
      command,
      rounds,
      warmup,
      binaries,
    }
  }
}
