use std::path::{Path, PathBuf};
use std::process::{Command as ChildCommand, Output};
use std::time::Duration;

use xrf_error::XrfError;
use xrf_utils::format_path;

use crate::commands::profile::run::profile_run_binary_report::ProfileRunBinaryReport;
use crate::commands::profile::run::rounds::RoundStatistics;
use crate::commands::profile::run::sample::SampledRun;
use crate::core::command::CommandResult;

/// One binary as the session measures it, and what its rounds have measured so far.
pub struct ProfileSubject {
  pub label: String,
  pub path: PathBuf,
  version: Vec<String>,
  runs: Vec<Duration>,
  peaks: Vec<u64>,
  means: Vec<u64>,
  exit_codes: Vec<i32>,
}

impl ProfileSubject {
  /// The binary at `path`, asked what it is, with room for `rounds` measurements.
  ///
  /// # Errors
  ///
  /// Returns an error when nothing is there to run, or it cannot be asked its version.
  pub fn new(path: PathBuf, rounds: usize) -> CommandResult<Self> {
    if !path.is_file() {
      return Err(XrfError::new_not_found_error(format!("No binary to measure at '{}'", format_path(&path))).into());
    }

    Ok(Self {
      label: Self::label_of(&path),
      version: Self::read_version(&path)?,
      path,
      runs: Vec::with_capacity(rounds),
      peaks: Vec::with_capacity(rounds),
      means: Vec::with_capacity(rounds),
      exit_codes: Vec::new(),
    })
  }

  /// Keeps one measured round.
  pub fn record(&mut self, measured: &SampledRun) {
    self.runs.push(measured.elapsed);

    if let Some(peak) = measured.peak_bytes {
      self.peaks.push(peak);
    }

    if let Some(mean) = measured.mean_bytes {
      self.means.push(mean);
    }

    if !self.exit_codes.contains(&measured.exit_code) {
      self.exit_codes.push(measured.exit_code);
    }
  }

  /// What the rounds came to, or `None` before any was measured.
  pub fn into_report(self) -> Option<ProfileRunBinaryReport> {
    let statistics: RoundStatistics = RoundStatistics::of(&self.runs, &self.peaks, &self.means)?;

    Some(ProfileRunBinaryReport::new(
      self.label,
      &self.path,
      self.version,
      self.runs,
      &statistics,
      self.exit_codes,
    ))
  }

  /// Names a binary by its file stem, disambiguated by its parent when two paths share one.
  ///
  /// Two builds of the same tool are both `xrf-cli`, and the parent directory is what a caller actually distinguished
  /// them by — `target/release` against a checkout of an older revision.
  fn label_of(path: &Path) -> String {
    let stem: String = path
      .file_stem()
      .map_or_else(|| String::from("binary"), |stem| stem.to_string_lossy().to_string());

    match path.parent().and_then(|parent| parent.file_name()) {
      Some(parent) => format!("{}/{stem}", parent.to_string_lossy()),
      None => stem,
    }
  }

  /// Asks a binary what it is, verbatim.
  ///
  /// `--version` rather than a report envelope's `build` block, so that a binary predating that field still identifies
  /// itself — which every historical revision worth measuring against does.
  fn read_version(path: &Path) -> CommandResult<Vec<String>> {
    let output: Output = ChildCommand::new(path).arg("--version").output().map_err(|error| {
      XrfError::new_io_error(format!("Failed to run '{}': {error}", format_path(path)), error.kind())
    })?;

    Ok(
      String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim_end().to_owned())
        .filter(|line| !line.is_empty())
        .collect(),
    )
  }
}
