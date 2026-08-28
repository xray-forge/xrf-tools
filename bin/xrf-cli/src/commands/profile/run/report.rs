use std::time::Duration;

use serde::Serialize;

use crate::commands::profile::run::rounds::RoundStatistics;

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfiledBinaryOutput {
  pub label: String,
  pub path: String,
  /// The binary's own `--version` long form, verbatim.
  ///
  /// Read from the binary rather than from the checkout, which is the trap this exists to close: a local build is
  /// routinely older than the source beside it, and only the binary knows which commit it came from. Taken from
  /// `--version` rather than from a report envelope's `build` block so that binaries predating that field — every
  /// historical revision worth comparing against — still identify themselves.
  pub version: Vec<String>,
  /// Every measured round in execution order, so a reader can see the spread rather than trust the median.
  #[serde(with = "xrf_utils::duration_ms_vec")]
  pub runs: Vec<Duration>,
  #[serde(with = "xrf_utils::duration_ms")]
  pub median: Duration,
  #[serde(with = "xrf_utils::duration_ms")]
  pub fastest: Duration,
  #[serde(with = "xrf_utils::duration_ms")]
  pub slowest: Duration,
  /// Exit codes observed, deduplicated.
  ///
  /// Reported because a command that started failing halfway through the session is measuring something other than the
  /// work it was asked to time, and a median hides that completely.
  pub exit_codes: Vec<i32>,
  /// Median against the first binary's, as a percentage. `None` for the first binary itself.
  ///
  /// Negative is faster. Only ever computed within one session, since two sessions cannot be compared.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub delta_percent: Option<f64>,
}

impl ProfiledBinaryOutput {
  pub fn new(
    label: String,
    path: String,
    version: Vec<String>,
    runs: Vec<Duration>,
    statistics: &RoundStatistics,
    exit_codes: Vec<i32>,
  ) -> Self {
    Self {
      label,
      path,
      version,
      runs,
      median: statistics.median,
      fastest: statistics.fastest,
      slowest: statistics.slowest,
      exit_codes,
      delta_percent: None,
    }
  }

  /// Records this binary's median against the session's first, which is the only comparison the protocol supports.
  pub fn compare_to(&mut self, baseline: Duration) {
    let baseline_ms: f64 = baseline.as_secs_f64();

    if baseline_ms > 0.0 {
      let ratio: f64 = (self.median.as_secs_f64() - baseline_ms) / baseline_ms * 100.0;

      // Two decimals: the spread on these trees is percent-scale, so more digits would imply a precision the
      // measurement does not have.
      self.delta_percent = Some((ratio * 100.0).round() / 100.0);
    }
  }
}
