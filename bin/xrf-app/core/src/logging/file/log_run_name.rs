use std::time::SystemTime;

use crate::logging::line::LogTimestamp;

/// A run of the application as its files are named: the UTC second it started and its process. Its log is
/// `xrf-app.<UTC start>.<process>.log`, such as `xrf-app.2026-11-22T10-30-05Z.4242.log`, and its UI log takes the same
/// name ending `.ui.log`, so a run is the pair. The start sorts as time does, and the process tells two runs of one
/// second apart; runs order by their start, then their process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct LogRunName {
  /// When the run started, to the second.
  start: LogTimestamp,
  process: u32,
}

impl LogRunName {
  const PREFIX: &'static str = "xrf-app.";
  const LOG_EXTENSION: &'static str = ".log";
  const UI_LOG_EXTENSION: &'static str = ".ui.log";

  /// The run that started at `start`, to its second, in the process `process`.
  pub(super) fn new(start: SystemTime, process: u32) -> Self {
    Self {
      start: LogTimestamp::of_system_time(start).floor_to_second(),
      process,
    }
  }

  /// The run whose log is named `name`, none where `name` is not a run's log: a run's UI log belongs to its run's, and
  /// the single `xrf-app.log` and `xrf-app.previous.log` earlier versions wrote are no run's.
  pub(super) fn parse_log(name: &str) -> Option<Self> {
    let (start, process) = name
      .strip_prefix(Self::PREFIX)?
      .strip_suffix(Self::LOG_EXTENSION)?
      .rsplit_once('.')?;
    let run: Self = Self {
      start: LogTimestamp::parse_file_stamp(start)?,
      process: process.parse().ok()?,
    };

    // Only the exact name a run writes is a run's, so pruning never takes a file this application did not name.
    (run.format_log() == name).then_some(run)
  }

  /// The name of the run's log.
  pub(super) fn format_log(&self) -> String {
    self.format_with(Self::LOG_EXTENSION)
  }

  /// The name of the run's UI log.
  pub(super) fn format_ui_log(&self) -> String {
    self.format_with(Self::UI_LOG_EXTENSION)
  }

  /// The run's name ending `extension`.
  fn format_with(&self, extension: &str) -> String {
    format!(
      "{}{}.{}{extension}",
      Self::PREFIX,
      self.start.format_file_stamp(),
      self.process
    )
  }
}
