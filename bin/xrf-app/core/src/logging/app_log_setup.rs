use std::io;

use xrf_logging::{LogFile, LogFilter, LogRun, LogSetup};

use crate::directories::AppDirectories;
use crate::logging::ui::UiLog;

/// How the application starts logging: the process's logger under its own name, and beside its log the run's UI log.
pub struct AppLogSetup;

impl AppLogSetup {
  /// What the application's log files are named by, `xrf-app.<UTC start>.<process>.log`.
  pub const LOG_NAME: &'static str = "xrf-app";

  /// What the run's UI log ends with, beside its log: `xrf-app.<UTC start>.<process>.ui.log`.
  const UI_LOG_SUFFIX: &'static str = ".ui.log";

  /// Installs the process's logger and answers this run's UI log.
  ///
  /// Records pass `RUST_LOG`, else the build's default level, and go to the console and to this run's log file, made
  /// under `appdata/logs` beside the executable or else in the local data directory of the bundle `identifier`. The UI
  /// log is closed where its file, or the run's log file, did not open.
  ///
  /// # Panics
  ///
  /// Where the process already has a logger: call it once, before anything logs, so nothing logged early is lost.
  pub fn install(identifier: &str, version: &str) -> UiLog {
    let filter: LogFilter = LogFilter::from_environment();
    let setup: LogSetup = LogSetup {
      application: Self::LOG_NAME,
      version: String::from(version),
      directories: AppDirectories::for_identifier(identifier).get_logs_directories(),
      tail: None,
    };

    match setup.install(&filter) {
      Some(run) => Self::open_ui_log(&run, version, &filter),
      None => UiLog::closed(),
    }
  }

  /// The UI log of `run`, filtered by `filter`, its opening logged; closed where its file did not open.
  fn open_ui_log(run: &LogRun, version: &str, filter: &LogFilter) -> UiLog {
    let file: io::Result<LogFile> = run.create_companion(Self::UI_LOG_SUFFIX);

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
}
