use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};

use log::{Level, Record};

/// The last records logged at or above a level, kept as short lines for a viewer inside the application, such as the
/// game's console, which takes them as they come. The oldest are let go once it is full; the log file keeps them all.
#[derive(Debug)]
pub struct LogTail {
  level: Level,
  capacity: usize,
  lines: Mutex<VecDeque<String>>,
}

impl LogTail {
  /// A tail of the records at `level` or above, holding at most `capacity` lines not yet taken.
  pub fn new(level: Level, capacity: usize) -> Self {
    Self {
      level,
      capacity,
      lines: Mutex::new(VecDeque::with_capacity(capacity)),
    }
  }

  /// The lines kept since the last take, oldest first.
  pub fn take(&self) -> Vec<String> {
    self
      .lines
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .drain(..)
      .collect()
  }

  /// Keeps `record` as a line, `LEVEL message`, where it is at the tail's level or above.
  pub(crate) fn keep(&self, record: &Record<'_>) {
    if record.level() > self.level || self.capacity == 0 {
      return;
    }

    let line: String = format!("{:<5} {}", record.level(), record.args());
    let mut lines = self.lines.lock().unwrap_or_else(PoisonError::into_inner);

    if lines.len() == self.capacity {
      lines.pop_front();
    }

    lines.push_back(line);
  }
}
