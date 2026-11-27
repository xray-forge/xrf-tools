use std::fmt;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use env_logger::{Builder, Logger, Target};
use log::{Level, LevelFilter, Log, Record};
use xrf_utils::wall_clock_millis;

use crate::line::log_layout::LogLayout;
use crate::line::log_timestamp::LogTimestamp;

#[derive(Clone, Default)]
struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

impl Write for SharedBuffer {
  fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
    self.0.lock().expect("the buffer").extend_from_slice(buffer);

    Ok(buffer.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

fn ui_record(level: Level, args: fmt::Arguments<'_>) -> Record<'_> {
  Record::builder()
    .level(level)
    .target("xrf_ui::levels")
    .args(args)
    .build()
}

#[test]
fn stamps_a_record_to_its_millisecond_and_names_a_run_to_its_second() {
  assert_eq!(LogTimestamp::of_unix_millis(0).to_string(), "1970-01-01T00:00:00.000Z");
  assert_eq!(
    LogTimestamp::of_unix_millis(1_709_210_096_009).to_string(),
    "2024-02-29T12:34:56.009Z"
  );
  assert_eq!(
    LogTimestamp::of_unix_millis(1_709_210_096_999).format_file_stamp(),
    "2024-02-29T12-34-56Z"
  );
  // A webview clock gone wrong must still give a line its stamp.
  assert_eq!(
    LogTimestamp::of_unix_millis(u64::MAX).to_string(),
    "9999-12-30T22:00:00.999Z"
  );
}

#[test]
fn reads_a_runs_stamp_back_to_its_second() {
  let moment: LogTimestamp = LogTimestamp::of_unix_millis(1_709_210_096_999);

  assert_eq!(
    LogTimestamp::parse_file_stamp("2024-02-29T12-34-56Z"),
    Some(moment.floor_to_second())
  );
  assert_eq!(moment.floor_to_second().to_string(), "2024-02-29T12:34:56.000Z");
  assert_eq!(LogTimestamp::parse_file_stamp("2024-02-29T12:34:56Z"), None);
  assert_eq!(LogTimestamp::parse_file_stamp("2023-02-29T12-34-56Z"), None);
}

// The backend's records and the UI's are read side by side, so they keep one layout, stamps included. env_logger stamps
// a record from the clock as it writes it, so the UI's line is written at each millisecond the write could have read.
#[test]
fn lays_a_ui_line_out_as_the_backends_logger_does() {
  let buffer: SharedBuffer = SharedBuffer::default();
  let mut builder: Builder = Builder::new();

  LogLayout::configure(&mut builder);

  let logger: Logger = builder
    .filter_level(LevelFilter::Trace)
    .target(Target::Pipe(Box::new(buffer.clone())))
    .build();

  for level in [Level::Error, Level::Warn, Level::Info, Level::Debug] {
    let before: u64 = wall_clock_millis();

    logger.log(&ui_record(level, format_args!("first\nsecond\n\nfourth")));

    let after: u64 = wall_clock_millis();
    let written: String = String::from_utf8(std::mem::take(&mut *buffer.0.lock().expect("the buffer"))).expect("text");

    assert!(
      (before..=after).any(|millis| {
        LogLayout::format_line(
          &ui_record(level, format_args!("first\nsecond\n\nfourth")),
          &LogTimestamp::of_unix_millis(millis),
        ) == written
      }),
      "at {level}, env_logger wrote {written:?}"
    );
  }
}
