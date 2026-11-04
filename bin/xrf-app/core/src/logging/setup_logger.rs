use std::panic;

use env_logger::{Builder, Target};
use log::LevelFilter;
use xrf_env_flags::EnvFlags;

use crate::logging::log_file::LogFile;
use crate::logging::log_sink::LogSink;

/// Configure environment logger, fallback to info level, writing to the console and to this run's log file, which is
/// kept beside the executable or in the data directory of the bundle `identifier`; its first line names the application
/// at `version`.
pub fn setup_logger(identifier: &str, version: &str) {
  let mut logger: Builder = env_logger::builder();

  if let Some(rust_log) = EnvFlags::RUST_LOG.read_text() {
    logger.parse_filters(&rust_log);
  } else {
    // Debug builds trace every command dispatch; release keeps the info lines the commands write, which is what makes
    // a user's log useful in a bug report.
    match cfg!(debug_assertions) {
      true => logger.filter_level(LevelFilter::Debug),
      false => logger.filter_level(LevelFilter::Info),
    };
  }

  let log_file: Result<LogFile, String> = LogFile::open(identifier);
  let (path, file) = match log_file {
    Ok(LogFile { path, file }) => (Ok(path), Some(file)),
    Err(refusals) => (Err(refusals), None),
  };

  logger.default_format();
  logger.target(Target::Pipe(Box::new(LogSink::new(file))));
  logger.init();

  match path {
    Ok(path) => log::info!("xrf-app {version} logging to '{}'", path.display()),
    Err(refusals) => log::warn!("xrf-app {version} logging to the console alone, no log file opening: {refusals}"),
  }

  log_panics();
}

/// Writes a panic to the log before the default hook reports it, which a release build without a console would lose.
fn log_panics() {
  let default = panic::take_hook();

  panic::set_hook(Box::new(move |info| {
    log::error!("Panicked: {info}");
    default(info);
  }));
}
