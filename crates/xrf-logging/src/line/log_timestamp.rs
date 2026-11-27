use std::fmt::{self, Display, Formatter};
use std::time::SystemTime;

use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::Offset;

/// A UTC moment as the log stamps a record, to the millisecond, and names a run, to the second. A moment past the last
/// the calendar holds, late in year 9999, reads as that last one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogTimestamp {
  instant: Timestamp,
}

impl LogTimestamp {
  /// A run's file name carries no colon, which Windows refuses in a name.
  const FILE_STAMP: &'static str = "%Y-%m-%dT%H-%M-%SZ";

  /// The moment `time` names.
  pub fn of_system_time(time: SystemTime) -> Self {
    Self {
      instant: Timestamp::try_from(time).unwrap_or(Timestamp::MAX),
    }
  }

  /// The moment `millis` milliseconds after the epoch, as the webview's clock gives it.
  pub fn of_unix_millis(millis: u64) -> Self {
    Self {
      instant: i64::try_from(millis)
        .ok()
        .and_then(|millis| Timestamp::from_millisecond(millis).ok())
        .unwrap_or(Timestamp::MAX),
    }
  }

  /// The moment a run's file name carries, `2026-11-22T10-30-05Z`, none where `stamp` names none.
  pub(crate) fn parse_file_stamp(stamp: &str) -> Option<Self> {
    DateTime::strptime(Self::FILE_STAMP, stamp)
      .and_then(|time| Offset::UTC.to_timestamp(time))
      .ok()
      .map(|instant| Self { instant })
  }

  /// The start of this moment's second, all a run's file name carries of it.
  pub(crate) fn floor_to_second(self) -> Self {
    Self {
      instant: Timestamp::from_second(self.instant.as_second()).unwrap_or(self.instant),
    }
  }

  /// The moment to the second as a run's file name carries it, `2026-11-22T10-30-05Z`.
  pub(crate) fn format_file_stamp(&self) -> String {
    self.instant.strftime(Self::FILE_STAMP).to_string()
  }
}

/// RFC 3339 to the millisecond, `2026-11-22T10:30:05.250Z`, as env_logger stamps a record at millisecond precision.
impl Display for LogTimestamp {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    write!(formatter, "{:.3}", self.instant)
  }
}
