use std::fs;
use std::path::{Path, PathBuf};

use log::Level;
use xrf_app_core::logging::UiLog;
use xrf_test_utils::GeneratedResources;

use crate::ui_log::{UiLogBatch, UiLogLevel, UiLogRecord};

const NEW_YEAR_2026_MILLIS: u64 = 1_767_225_600_000;

fn ui_log_path(name: &str) -> PathBuf {
  let directory: PathBuf = GeneratedResources::path(&format!("ui-log/{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("a test folder");

  directory.join("xrf-app.ui.log")
}

fn read_lines_after_first(path: &Path) -> Vec<String> {
  fs::read_to_string(path)
    .expect("the UI log reads")
    .lines()
    .skip(1)
    .map(String::from)
    .collect()
}

fn record(level: UiLogLevel, scope: &str, millis: u64, message: &str) -> UiLogRecord {
  UiLogRecord {
    level,
    scope: String::from(scope),
    time: NEW_YEAR_2026_MILLIS + millis,
    message: String::from(message),
  }
}

#[test]
fn a_forwarded_record_reaches_the_ui_log_under_its_scope_at_its_own_time() {
  let path: PathBuf = ui_log_path("forwarded");
  let log: UiLog = UiLog::create_fixture(&path, "info").expect("the UI log opens");
  let records: Vec<UiLogRecord> = serde_json::from_value(serde_json::json!([
    { "level": "error", "scope": "levels", "time": NEW_YEAR_2026_MILLIS + 61_500, "message": "Failed\n  at open" },
    { "level": "info", "scope": "textures", "time": NEW_YEAR_2026_MILLIS + 2_000, "message": "Opened" },
  ]))
  .expect("a batch as the webview sends it");

  UiLogBatch::new(records)
    .and_then(|batch| batch.write_to(&log))
    .expect("the batch is written");

  assert_eq!(
    read_lines_after_first(&path),
    [
      "[2026-01-01T00:01:01.500Z ERROR xrf_ui::levels] Failed",
      "      at open",
      "[2026-01-01T00:00:02.000Z INFO  xrf_ui::textures] Opened",
    ]
  );
}

#[test]
fn a_record_below_the_filter_is_not_written() {
  let path: PathBuf = ui_log_path("filtered");
  let log: UiLog = UiLog::create_fixture(&path, "debug,xrf_ui=warn").expect("the UI log opens");

  UiLogBatch::new(vec![
    record(UiLogLevel::Debug, "levels", 0, "hidden"),
    record(UiLogLevel::Info, "levels", 0, "hidden"),
    record(UiLogLevel::Warn, "levels", 0, "shown"),
  ])
  .and_then(|batch| batch.write_to(&log))
  .expect("the batch is written");

  assert_eq!(
    read_lines_after_first(&path),
    ["[2026-01-01T00:00:00.000Z WARN  xrf_ui::levels] shown"]
  );
}

#[test]
fn a_batch_over_the_limit_is_refused_whole() {
  let records: Vec<UiLogRecord> = (0..=UiLogBatch::MAX_RECORDS as u64)
    .map(|index| record(UiLogLevel::Error, "flood", index, "line"))
    .collect();

  assert_eq!(
    UiLogBatch::new(records).err(),
    Some(String::from(
      "A UI log batch of 513 records is over the 512 one call takes"
    ))
  );
}

#[test]
fn a_full_batch_is_written_whole() {
  let path: PathBuf = ui_log_path("full");
  let log: UiLog = UiLog::create_fixture(&path, "debug").expect("the UI log opens");
  let records: Vec<UiLogRecord> = (0..UiLogBatch::MAX_RECORDS as u64)
    .map(|index| record(UiLogLevel::Error, "flood", index, "line"))
    .collect();

  UiLogBatch::new(records)
    .and_then(|batch| batch.write_to(&log))
    .expect("a full batch is taken");

  assert_eq!(read_lines_after_first(&path).len(), UiLogBatch::MAX_RECORDS);
}

#[test]
fn a_closed_log_takes_a_batch_and_writes_nothing() {
  let batch: UiLogBatch =
    UiLogBatch::new(vec![record(UiLogLevel::Error, "levels", 0, "lost")]).expect("a batch within the limit");

  assert_eq!(batch.write_to(&UiLog::closed()), Ok(()));
}

#[test]
fn the_level_answered_is_one_the_webview_logs_at() {
  assert_eq!(UiLogLevel::from(Level::Trace), UiLogLevel::Debug);
  assert_eq!(UiLogLevel::from(Level::Warn), UiLogLevel::Warn);
  assert_eq!(Level::from(UiLogLevel::Error), Level::Error);
  assert_eq!(
    serde_json::to_value([UiLogLevel::Error, UiLogLevel::Warn, UiLogLevel::Info, UiLogLevel::Debug])
      .expect("levels serialize"),
    serde_json::json!(["error", "warn", "info", "debug"])
  );
}
