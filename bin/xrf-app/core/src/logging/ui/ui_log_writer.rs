use std::fs::File;
use std::io::{self, Write};
use std::sync::{Mutex, PoisonError};

/// The open UI log file, written a whole line at a time.
pub(super) struct UiLogWriter {
  file: Mutex<File>,
}

impl UiLogWriter {
  /// A writer to `file`.
  pub(super) fn new(file: File) -> Self {
    Self { file: Mutex::new(file) }
  }

  /// Writes `line`, laid out whole, to the file in a single call, so no other record's line splits it.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the file refuses the line.
  pub(super) fn write(&self, line: &str) -> io::Result<()> {
    self
      .file
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .write_all(line.as_bytes())
  }
}
