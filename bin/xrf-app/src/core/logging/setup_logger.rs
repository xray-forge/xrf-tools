use std::panic;

use env_logger::{Builder, Target};
use log::LevelFilter;
use xrf_env_flags::EnvFlags;

use crate::core::logging::log_file::LogFile;
use crate::core::logging::log_sink::LogSink;

/// Configure environment logger, fallback to info level, writing to the console and to this run's log file, which is
/// kept beside the executable or in the data directory of the bundle `identifier`.
pub fn setup_logger(identifier: &str) {
  let mut logger: Builder = env_logger::builder();

  if let Some(rust_log) = EnvFlags::RUST_LOG.read_text() {
    logger.parse_filters(&rust_log);
  } else {
    // Debug builds trace every command dispatch; release keeps the info lines the commands already
    // write, which is what makes a user's log useful in a bug report. At the previous warn/error
    // levels none of them were ever emitted, so the logging in the commands did nothing at all.
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
    Ok(path) => log::info!("xrf-app {} logging to '{}'", env!("CARGO_PKG_VERSION"), path.display()),
    Err(refusals) => log::warn!(
      "xrf-app {} logging to the console alone, no log file opening: {refusals}",
      env!("CARGO_PKG_VERSION")
    ),
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
