use std::fs::File;
use std::io::{self, Write};

/// Where every log record goes: the console, and the log file where one opened. A record is written to the file as it
/// comes, unbuffered, so a log read after a freeze or a crash holds everything before it.
pub struct LogSink {
  file: Option<File>,
}

impl LogSink {
  pub fn new(file: Option<File>) -> Self {
    Self { file }
  }
}

impl Write for LogSink {
  fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
    // A release build has no console, so the file's write alone may fail the record.
    let _ = io::stderr().write_all(buffer);

    if let Some(file) = &mut self.file {
      file.write_all(buffer)?;
    }

    Ok(buffer.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    let _ = io::stderr().flush();

    match &mut self.file {
      Some(file) => file.flush(),
      None => Ok(()),
    }
  }
}
