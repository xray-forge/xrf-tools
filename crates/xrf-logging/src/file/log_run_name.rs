use std::time::SystemTime;

use crate::line::log_timestamp::LogTimestamp;

/// A run of an application as its files are named: the application, the UTC second it started and its process. Its
/// log is `<application>.<UTC start>.<process>.log`, such as `xrf-app.2026-11-22T10-30-05Z.4242.log`, and a companion
/// file takes the same name with its own suffix before `.log`, such as the app's `.ui.log`, so a run is its files
/// together. The start sorts as time does, and the process tells two runs of one second apart; runs order by their
/// start, then their process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LogRunName {
  application: &'static str,
  /// When the run started, to the second.
  start: LogTimestamp,
  process: u32,
}

impl LogRunName {
  pub(crate) const LOG_EXTENSION: &'static str = ".log";

  /// The run of `application` that started at `start`, to its second, in the process `process`.
  pub(crate) fn new(application: &'static str, start: SystemTime, process: u32) -> Self {
    Self {
      application,
      start: LogTimestamp::of_system_time(start).floor_to_second(),
      process,
    }
  }

  /// The run of `application` whose log is named `name`, none where `name` is not a run's log: a companion file belongs
  /// to its run's, and the single `<application>.log` and `<application>.previous.log` earlier versions wrote are no
  /// run's.
  pub(crate) fn parse_log(application: &'static str, name: &str) -> Option<Self> {
    let (start, process) = name
      .strip_prefix(application)?
      .strip_prefix('.')?
      .strip_suffix(Self::LOG_EXTENSION)?
      .rsplit_once('.')?;
    let run: Self = Self {
      application,
      start: LogTimestamp::parse_file_stamp(start)?,
      process: process.parse().ok()?,
    };

    // Only the exact name a run writes is a run's, so pruning never takes a file this application did not name.
    (run.format_log() == name).then_some(run)
  }

  /// The application the run is one of, its files' first word.
  pub(crate) fn get_application(&self) -> &'static str {
    self.application
  }

  /// The name of the run's log.
  pub(crate) fn format_log(&self) -> String {
    format!("{}{}", self.format_stem(), Self::LOG_EXTENSION)
  }

  /// The name of the run's companion file ending `suffix`, such as `.ui.log`.
  pub(crate) fn format_companion(&self, suffix: &str) -> String {
    format!("{}{suffix}", self.format_stem())
  }

  /// Whether `name` is the run's log or one of its companions: its stem, then a suffix ending `.log`.
  pub(crate) fn is_own_file(&self, name: &str) -> bool {
    name
      .strip_prefix(&self.format_stem())
      .is_some_and(|suffix| suffix.starts_with('.') && suffix.ends_with(Self::LOG_EXTENSION))
  }

  /// What every file of the run starts with: `<application>.<UTC start>.<process>`.
  fn format_stem(&self) -> String {
    format!(
      "{}.{}.{}",
      self.application,
      self.start.format_file_stamp(),
      self.process
    )
  }
}
