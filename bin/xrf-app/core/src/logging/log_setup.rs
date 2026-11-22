use std::fs::File;
use std::{io, panic};

use env_logger::{Builder, Target};

use crate::logging::file::{LogFile, LogRun};
use crate::logging::line::LogLayout;
use crate::logging::log_filter::LogFilter;
use crate::logging::log_sink::LogSink;
use crate::logging::ui::UiLog;

/// How the process starts logging: its logger, writing to the console and to this run's log file, panics included, and
/// beside that file the run's UI log.
pub struct LogSetup;

impl LogSetup {
  /// Installs the process's logger and answers this run's UI log.
  ///
  /// Records pass `RUST_LOG`, else the build's default level, and go to the console and to this run's log file, made
  /// under `appdata` beside the executable or else in the local data directory of the bundle `identifier`. The first
  /// line names the application at `version` and the file, or why no file opened. The UI log is closed where its file,
  /// or the run's log file, did not open.
  ///
  /// # Panics
  ///
  /// Where the process already has a logger: call it once, before anything logs, so nothing logged early is lost.
  pub fn install(identifier: &str, version: &str) -> UiLog {
    let filter: LogFilter = LogFilter::from_environment();

    match LogRun::open(identifier) {
      Ok(LogRun { log, ui_log }) => {
        Self::start(&filter, Some(log.file));
        log::info!("xrf-app {version} logging to '{}'", log.path.display());

        Self::open_ui_log(ui_log, version, &filter)
      }
      Err(refusals) => {
        Self::start(&filter, None);
        log::warn!("xrf-app {version} logging to the console alone, no log file opening: {refusals}");

        UiLog::closed()
      }
    }
  }

  /// Installs the process's logger, filtered by `filter`, writing to the console and to `file` where the run has one,
  /// and has it log panics.
  fn start(filter: &LogFilter, file: Option<File>) {
    let mut logger: Builder = env_logger::builder();

    filter.configure(&mut logger);
    LogLayout::configure(&mut logger);
    logger.target(Target::Pipe(Box::new(LogSink::new(file))));
    logger.init();

    Self::log_panics();
  }

  /// The UI log writing to `file`, filtered by `filter`, its opening logged; closed where `file` did not open.
  fn open_ui_log(file: io::Result<LogFile>, version: &str, filter: &LogFilter) -> UiLog {
    match file {
      Ok(file) => {
        log::info!("UI records logging to '{}'", file.path.display());

        UiLog::open(file, version, filter)
      }
      Err(error) => {
        log::warn!("UI records logging nowhere, no UI log file opening: {error}");

        UiLog::closed()
      }
    }
  }

  /// Writes a panic to the log before the default hook reports it, which a release build without a console would lose.
  fn log_panics() {
    let default = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
      log::error!("Panicked: {info}");
      default(info);
    }));
  }
}
