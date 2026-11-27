use log::{Level, Record};

use crate::tail::log_tail::LogTail;

fn keep(tail: &LogTail, level: Level, message: &str) {
  tail.keep(&Record::builder().level(level).args(format_args!("{message}")).build());
}

#[test]
fn keeps_records_at_its_level_or_above_as_lines_taken_once() {
  let tail: LogTail = LogTail::new(Level::Info, 10);

  keep(&tail, Level::Debug, "a buffer allocated");
  keep(&tail, Level::Info, "level opened");
  keep(&tail, Level::Error, "a sector failed");

  assert_eq!(tail.take(), vec!["INFO  level opened", "ERROR a sector failed"]);
  assert!(tail.take().is_empty());
}

#[test]
fn lets_the_oldest_go_once_full() {
  let tail: LogTail = LogTail::new(Level::Info, 2);

  for message in ["one", "two", "three"] {
    keep(&tail, Level::Warn, message);
  }

  assert_eq!(tail.take(), vec!["WARN  two", "WARN  three"]);
}
