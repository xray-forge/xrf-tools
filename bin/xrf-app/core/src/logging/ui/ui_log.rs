use std::fmt::Arguments;
use std::io;
use std::time::SystemTime;

use log::{Level, Record};

use crate::logging::file::LogFile;
use crate::logging::line::{LogLayout, LogTimestamp};
use crate::logging::log_filter::LogFilter;
use crate::logging::ui::UiLogEntry;
use crate::logging::ui::ui_log_filter::UiLogFilter;
use crate::logging::ui::ui_log_writer::UiLogWriter;

/// This run's UI log: the webview's records, written to the run's UI log file alone, never to the console nor through
/// the process's logger. A record goes under the target `xrf_ui::<scope>`, passes the backend's filter rules, and is
/// laid out as a backend line stamped with the millisecond the webview logged it. Each is written as it comes,
/// unbuffered, as the backend's are. A run whose log or UI log file did not open has a closed one ([`Self::closed`]),
/// which writes nothing.
pub struct UiLog {
  open: Option<OpenUiLog>,
}

/// An open UI log: its file and the filter its records pass.
struct OpenUiLog {
  filter: UiLogFilter,
  writer: UiLogWriter,
}

impl UiLog {
  /// The target every UI record is written under, each scope a module of it, so `RUST_LOG=xrf_ui=debug` selects them.
  pub const TARGET: &'static str = "xrf_ui";

  /// What every scoped record's target starts with: the target and the module separator.
  pub(super) const SCOPE_PREFIX: &'static str = "xrf_ui::";

  /// The log writing to `log`, this run's UI log file, with records `rules` let through; its first line names the
  /// application at `version`, whatever the rules.
  pub(crate) fn open(log: LogFile, version: &str, rules: &LogFilter) -> Self {
    let LogFile { path, file } = log;
    let writer: UiLogWriter = UiLogWriter::new(file);

    let _ = writer.write(&LogLayout::format_line(
      &Record::builder()
        .level(Level::Info)
        .target(Self::TARGET)
        .args(format_args!("xrf-app {version} UI logging to '{}'", path.display()))
        .build(),
      &LogTimestamp::of_system_time(SystemTime::now()),
    ));

    Self {
      open: Some(OpenUiLog {
        filter: UiLogFilter::new(rules),
        writer,
      }),
    }
  }

  /// A log of no file, as a run whose log or UI log file did not open has: it writes nothing and lets no level through.
  pub fn closed() -> Self {
    Self { open: None }
  }

  /// A log writing to a new file at `path`, filtered by `filters` as `RUST_LOG` spells them.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the file cannot be made.
  #[cfg(any(test, feature = "fixtures"))]
  pub fn create_fixture(path: &std::path::Path, filters: &str) -> io::Result<Self> {
    Ok(Self::open(
      LogFile::create(path.to_path_buf())?,
      "fixture",
      &LogFilter::from_directives(filters),
    ))
  }

  /// The most verbose level at which a record under [`Self::TARGET`] or any of its scopes can be written, none where no
  /// such record can be. The webview drops a record below it before sending, which loses only records the backend
  /// would drop whatever their scope. It may name a level some scopes are not written at: [`Self::write`] filters each
  /// record again and decides.
  pub fn get_level(&self) -> Option<Level> {
    self.open.as_ref().and_then(|open| open.filter.get_level())
  }

  /// Writes `entry` where the filter lets it through, in one write.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the file refuses the line.
  pub fn write(&self, entry: &UiLogEntry<'_>) -> io::Result<()> {
    let Some(OpenUiLog { filter, writer }) = &self.open else {
      return Ok(());
    };
    let target: String = match entry.scope.is_empty() {
      true => String::from(Self::TARGET),
      false => format!("{}{}", Self::SCOPE_PREFIX, entry.scope),
    };
    let message: Arguments<'_> = format_args!("{}", entry.message);
    let record: Record<'_> = Record::builder()
      .level(entry.level)
      .target(&target)
      .args(message)
      .build();

    if !filter.matches(&record) {
      return Ok(());
    }

    writer.write(&LogLayout::format_line(
      &record,
      &LogTimestamp::of_unix_millis(entry.time),
    ))
  }
}
