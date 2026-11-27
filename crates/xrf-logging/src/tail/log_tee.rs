use std::sync::Arc;

use env_logger::Logger;
use log::{Log, Metadata, Record};

use crate::tail::log_tail::LogTail;

/// The process's logger with a tail: every record the logger takes is written as it writes it and kept by the tail.
pub(crate) struct LogTee {
  pub(crate) logger: Logger,
  pub(crate) tail: Arc<LogTail>,
}

impl Log for LogTee {
  fn enabled(&self, metadata: &Metadata<'_>) -> bool {
    self.logger.enabled(metadata)
  }

  fn log(&self, record: &Record<'_>) {
    if self.logger.matches(record) {
      self.logger.log(record);
      self.tail.keep(record);
    }
  }

  fn flush(&self) {
    self.logger.flush();
  }
}
