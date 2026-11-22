use std::time::{SystemTime, UNIX_EPOCH};

/// A run's log file name, `xrf-app.<UTC start>.<process>.log`, such as `xrf-app.2026-11-22T10-30-05Z.4242.log`: the
/// start to the second sorts as time does, and the process tells two runs of one second apart.
pub(super) struct LogRunName;

impl LogRunName {
  const PREFIX: &'static str = "xrf-app.";
  const EXTENSION: &'static str = ".log";

  const SECONDS_IN_DAY: i64 = 86_400;

  /// The log name of the run that started at `time` in the process `process`.
  pub(super) fn format(time: SystemTime, process: u32) -> String {
    let seconds: i64 = time
      .duration_since(UNIX_EPOCH)
      .map_or(0, |since| i64::try_from(since.as_secs()).unwrap_or(i64::MAX));
    let (year, month, day) = Self::to_civil(seconds.div_euclid(Self::SECONDS_IN_DAY));
    let second: i64 = seconds.rem_euclid(Self::SECONDS_IN_DAY);
    let (hour, minute, second) = (second / 3_600, second / 60 % 60, second % 60);

    format!(
      "{}{year:04}-{month:02}-{day:02}T{hour:02}-{minute:02}-{second:02}Z.{process}{}",
      Self::PREFIX,
      Self::EXTENSION
    )
  }

  /// Whether `name` is a run's log name: not the single `xrf-app.log` and `xrf-app.previous.log` earlier versions wrote.
  pub(super) fn is_run(name: &str) -> bool {
    name
      .strip_prefix(Self::PREFIX)
      .and_then(|rest| rest.strip_suffix(Self::EXTENSION))
      .is_some_and(|run| run.starts_with(|first: char| first.is_ascii_digit()))
  }

  /// The year, month and day of the day `days` after 1970-01-01 in the proleptic Gregorian calendar: Howard Hinnant's
  /// `civil_from_days`, in eras of 400 years that start on 1 March.
  fn to_civil(days: i64) -> (i64, i64, i64) {
    let shifted: i64 = days + 719_468;
    let era: i64 = shifted.div_euclid(146_097);
    let day_of_era: i64 = shifted.rem_euclid(146_097);
    let year_of_era: i64 = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year: i64 = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march: i64 = (5 * day_of_year + 2) / 153;
    let day: i64 = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month: i64 = (month_from_march + 2) % 12 + 1;
    let year: i64 = year_of_era + era * 400 + i64::from(month <= 2);

    (year, month, day)
  }
}
