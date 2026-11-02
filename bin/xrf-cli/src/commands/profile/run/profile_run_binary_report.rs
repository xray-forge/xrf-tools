use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use crate::commands::profile::run::rounds::RoundStatistics;

/// What one binary of a profiling session measured.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRunBinaryReport {
  label: String,
  path: String,
  /// The binary's own `--version` long form, verbatim.
  ///
  /// Read from the binary rather than from the checkout, which is the trap this exists to close: a local build is
  /// routinely older than the source beside it, and only the binary knows which commit it came from. Taken from
  /// `--version` rather than from a report envelope's `build` block so that binaries predating that field — every
  /// historical revision worth comparing against — still identify themselves.
  version: Vec<String>,
  /// Every measured round in execution order, so a reader can see the spread rather than trust the median.
  #[serde(with = "xrf_utils::duration_ms_vec")]
  runs: Vec<Duration>,
  #[serde(with = "xrf_utils::duration_ms")]
  median: Duration,
  #[serde(with = "xrf_utils::duration_ms")]
  fastest: Duration,
  #[serde(with = "xrf_utils::duration_ms")]
  slowest: Duration,
  /// Largest resident set seen during a round, as the median of the per-round peaks.
  #[serde(skip_serializing_if = "Option::is_none")]
  peak_bytes: Option<u64>,
  /// Mean resident set across a round's samples, as the median of the per-round means.
  #[serde(skip_serializing_if = "Option::is_none")]
  mean_bytes: Option<u64>,
  /// Exit codes observed, deduplicated.
  ///
  /// Reported because a command that started failing halfway through the session is measuring something other than the
  /// work it was asked to time, and a median hides that completely.
  exit_codes: Vec<i32>,
  /// Median against the first binary's, as a percentage. `None` for the first binary itself.
  ///
  /// Negative is faster. Only ever computed within one session, since two sessions cannot be compared.
  #[serde(skip_serializing_if = "Option::is_none")]
  delta_percent: Option<f64>,
}

impl ProfileRunBinaryReport {
  pub fn new(
    label: String,
    path: &Path,
    version: Vec<String>,
    runs: Vec<Duration>,
    statistics: &RoundStatistics,
    exit_codes: Vec<i32>,
  ) -> Self {
    Self {
      label,
      path: xrf_utils::to_portable_path_string(path),
      version,
      runs,
      median: statistics.median,
      fastest: statistics.fastest,
      slowest: statistics.slowest,
      peak_bytes: statistics.peak_bytes,
      mean_bytes: statistics.mean_bytes,
      exit_codes,
      delta_percent: None,
    }
  }

  pub fn median(&self) -> Duration {
    self.median
  }

  /// Records this binary's median against the session's first, which is the only comparison the protocol supports.
  pub fn compare_to(&mut self, baseline: Duration) {
    let baseline_seconds: f64 = baseline.as_secs_f64();

    if baseline_seconds > 0.0 {
      let ratio: f64 = (self.median.as_secs_f64() - baseline_seconds) / baseline_seconds * 100.0;

      // Two decimals: the spread on these trees is percent-scale, so more digits would imply a precision the
      // measurement does not have.
      self.delta_percent = Some((ratio * 100.0).round() / 100.0);
    }
  }

  /// The binary's median as one line, with its difference from the baseline and its memory where they were measured.
  pub fn describe(&self) -> String {
    let delta: String = self
      .delta_percent
      .map_or_else(String::new, |delta| format!(" ({delta:+.2}%)"));

    let memory: String = match (self.peak_bytes, self.mean_bytes) {
      (Some(peak), Some(mean)) => format!(
        ", peak {} / mean {}",
        xrf_utils::format_bytes(peak),
        xrf_utils::format_bytes(mean)
      ),
      // Absent together: both come from the same samples, and a command that finished inside one interval has none.
      _ => String::new(),
    };

    format!(
      "{}: median {}{delta}{memory}",
      self.label,
      xrf_utils::format_duration(self.median)
    )
  }
}
