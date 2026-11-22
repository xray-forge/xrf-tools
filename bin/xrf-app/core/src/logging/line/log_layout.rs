use env_logger::Builder;
use log::Record;

use crate::logging::line::LogTimestamp;

/// The layout every log line takes, the backend's and the UI log's alike, so the two files read side by side:
/// env_logger's default format, stamped to the millisecond.
pub(crate) struct LogLayout;

impl LogLayout {
  /// What starts each further line of a record, as env_logger's default format indents it.
  const CONTINUATION: &'static str = "\n    ";

  /// Lays `logger`'s records out in env_logger's default format with millisecond stamps, so records a burst writes
  /// within one second stay ordered by their stamps.
  pub(crate) fn configure(logger: &mut Builder) {
    logger.default_format().format_timestamp_millis();
  }

  /// The line [`Self::configure`] has env_logger write for `record` at `timestamp`:
  /// `[<time to the millisecond> <level> <target>] <text>`, each further line of the text indented, without styles, as
  /// a file target gets it.
  pub(crate) fn format_line(record: &Record<'_>, timestamp: &LogTimestamp) -> String {
    let text: String = record.args().to_string();
    let mut line: String = format!("[{timestamp} {:<5} {}] ", record.level(), record.target());

    line.push_str(&text.replace('\n', Self::CONTINUATION));
    line.push('\n');
    line
  }
}
