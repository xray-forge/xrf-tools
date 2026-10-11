use env_logger::Builder;
use log::LevelFilter;
use xrf_env_flags::EnvFlags;

/// Configures logging from `RUST_LOG`, or falls back to warnings in debug builds and errors in release builds.
pub fn setup_logger() {
  let mut logger: Builder = env_logger::builder();

  if let Some(rust_log) = EnvFlags::RUST_LOG.read_text() {
    logger.parse_filters(&rust_log);
  } else {
    match cfg!(debug_assertions) {
      true => logger.filter_level(LevelFilter::Warn),
      false => logger.filter_level(LevelFilter::Error),
    };
  }

  logger.default_format();
  logger.init();
}
