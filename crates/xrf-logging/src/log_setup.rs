use std::fs::File;
use std::panic;
use std::path::PathBuf;

use env_logger::{Builder, Target};

use crate::file::log_run::LogRun;
use crate::line::log_layout::LogLayout;
use crate::log_filter::LogFilter;
use crate::log_sink::LogSink;

/// How a process starts logging: its logger, writing to the console and to this run's log file, panics included.
pub struct LogSetup {
  /// The application's name, which its log files are named by and its first line says, such as `xrf-app`.
  pub application: &'static str,
  pub version: String,
  /// Where the run's log may be made, best first.
  pub directories: Vec<PathBuf>,
}

impl LogSetup {
  /// Installs the process's logger and answers this run, none where no directory took its log.
  ///
  /// Records pass `filter` and go to the console and to this run's log file. The first line names the application at
  /// its version and the file, or why no file opened.
  ///
  /// # Panics
  ///
  /// Where the process already has a logger: call it once, before anything logs, so nothing logged early is lost.
  pub fn install(self, filter: &LogFilter) -> Option<LogRun> {
    let Self {
      application,
      version,
      directories,
    } = self;

    match LogRun::open(application, &directories) {
      Ok(run) => match run.log.file.try_clone() {
        Ok(file) => {
          Self::start(filter, Some(file));
          log::info!("{application} {version} logging to '{}'", run.log.path.display());

          Some(run)
        }
        Err(error) => {
          Self::start(filter, None);
          log::warn!(
            "{application} {version} logging to the console alone, '{}' not writable: {error}",
            run.log.path.display()
          );

          None
        }
      },
      Err(refusals) => {
        Self::start(filter, None);
        log::warn!("{application} {version} logging to the console alone, no log file opening: {refusals}");

        None
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

  /// Writes a panic to the log before the default hook reports it, which a release build without a console would lose.
  fn log_panics() {
    let default = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
      log::error!("Panicked: {info}");
      default(info);
    }));
  }
}
