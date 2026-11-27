use std::fs;
use std::path::{Path, PathBuf};

use env_logger::{Builder, Logger};
use log::{Level, Log, Metadata};
use xrf_logging::LogFilter;
use xrf_test_utils::GeneratedResources;

use crate::logging::ui::ui_log_filter::UiLogFilter;
use crate::logging::{UiLog, UiLogEntry};

const NEW_YEAR_2026: u64 = 1_767_225_600;

fn ui_log_path(case: &str) -> PathBuf {
  let directory: PathBuf = GeneratedResources::path(&format!("log_file/{case}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("the folder is made");

  directory.join("xrf-app.ui.log")
}

fn read_records(path: &Path) -> Vec<String> {
  let text: String = fs::read_to_string(path).expect("the UI log reads");
  let mut lines = text.split_inclusive('\n');

  // `[2026-01-01T00:00:05.250Z INFO  xrf_ui] ...`: stamped to the millisecond, as every record is.
  assert!(
    lines.next().is_some_and(|first| {
      first.get(20..21) == Some(".")
        && first
          .get(24..)
          .is_some_and(|rest| rest.starts_with("Z INFO  xrf_ui] xrf-app fixture UI logging to '"))
    }),
    "the first line names the application: {text:?}"
  );

  // A record's further lines start indented, so each record is one entry here.
  let mut records: Vec<String> = Vec::new();

  for line in lines {
    match line.starts_with(' ') {
      true => records.last_mut().expect("a record to continue").push_str(line),
      false => records.push(String::from(line)),
    }
  }

  records
}

fn time_in_2026(second: u64, millis: u64) -> u64 {
  (NEW_YEAR_2026 + second) * 1_000 + millis
}

#[test]
fn writes_a_record_under_its_scope_stamped_when_the_webview_logged_it() {
  let path: PathBuf = ui_log_path("ui_record");
  let log: UiLog = UiLog::create_fixture(&path, "debug").expect("the UI log opens");

  log
    .write(&UiLogEntry {
      level: Level::Warn,
      scope: "levels/viewer",
      time: time_in_2026(5, 999),
      message: "Cannot draw\n  at draw (viewer.ts:12)",
    })
    .expect("the record is written");
  log
    .write(&UiLogEntry {
      level: Level::Debug,
      scope: "",
      time: time_in_2026(3, 0),
      message: "unscoped",
    })
    .expect("the record is written");

  assert_eq!(
    read_records(&path),
    [
      "[2026-01-01T00:00:05.999Z WARN  xrf_ui::levels/viewer] Cannot draw\n      at draw (viewer.ts:12)\n",
      "[2026-01-01T00:00:03.000Z DEBUG xrf_ui] unscoped\n",
    ]
  );
}

// Every UI log file says which application wrote it, even one its rules keep every record out of.
#[test]
fn names_the_application_whatever_the_filter() {
  let path: PathBuf = ui_log_path("ui_heading");
  let log: UiLog = UiLog::create_fixture(&path, "off").expect("the UI log opens");

  log
    .write(&UiLogEntry {
      level: Level::Error,
      scope: "levels",
      time: time_in_2026(0, 0),
      message: "dropped",
    })
    .expect("the record is dropped");

  assert!(read_records(&path).is_empty(), "the heading alone");
  assert_eq!(log.get_level(), None);
}

#[test]
fn writes_no_record_its_filter_drops() {
  let path: PathBuf = ui_log_path("ui_filtered");
  let log: UiLog = UiLog::create_fixture(&path, "debug,xrf_ui=warn,xrf_ui::verbose=debug").expect("the UI log opens");

  for (level, scope) in [
    (Level::Info, "quiet"),
    (Level::Error, "quiet"),
    (Level::Debug, "verbose"),
  ] {
    log
      .write(&UiLogEntry {
        level,
        scope,
        time: time_in_2026(0, 0),
        message: "text",
      })
      .expect("the record is written or dropped");
  }

  assert_eq!(
    read_records(&path),
    [
      "[2026-01-01T00:00:00.000Z ERROR xrf_ui::quiet] text\n",
      "[2026-01-01T00:00:00.000Z DEBUG xrf_ui::verbose] text\n",
    ]
  );
  assert_eq!(
    log.get_level(),
    Some(Level::Debug),
    "the scope's level, above the target's own"
  );
}

// The webview drops what falls below the answer, so a scope named more verbosely than the target must lift it.
#[test]
fn answers_the_level_a_scope_named_more_verbosely_than_the_target_is_written_at() {
  let path: PathBuf = ui_log_path("ui_scope_level");
  let log: UiLog = UiLog::create_fixture(&path, "xrf_ui=warn,xrf_ui::views=debug").expect("the UI log opens");

  assert_eq!(log.get_level(), Some(Level::Debug));

  for scope in ["views", "levels"] {
    log
      .write(&UiLogEntry {
        level: Level::Debug,
        scope,
        time: time_in_2026(0, 0),
        message: "text",
      })
      .expect("the record is written or dropped");
  }

  assert_eq!(
    read_records(&path),
    ["[2026-01-01T00:00:00.000Z DEBUG xrf_ui::views] text\n"],
    "the named scope's debug record alone"
  );
}

#[test]
fn answers_the_level_the_ui_target_is_written_at() {
  let path: PathBuf = ui_log_path("ui_level");

  for (filters, level) in [
    ("info", Some(Level::Info)),
    ("trace", Some(Level::Debug)),
    ("info,xrf_ui=error", Some(Level::Error)),
    ("error,xrf_ui::views=info/regex", Some(Level::Info)),
    ("error,xrf_ui:=debug", Some(Level::Debug)),
    ("xrf_ui=info,xrf_app::views=debug", Some(Level::Info)),
    ("xrf_app=debug", None),
    ("debug,xrf_ui=off", None),
  ] {
    let log: UiLog = UiLog::create_fixture(&path, filters).expect("the UI log opens");

    assert_eq!(log.get_level(), level, "under '{filters}'");
  }

  assert_eq!(UiLog::closed().get_level(), None, "nothing is written without a file");
}

// The answer reads the directives' names by its own cut of them, so it is held to the real filter's answer for a
// record of every scope a directive names, whatever the directive's spelling.
#[test]
fn answers_the_level_the_backends_logger_writes_some_named_scope_at() {
  let cases: [(&str, &[&str]); 12] = [
    ("xrf_ui=warn, xrf_ui::views=debug", &["views"]),
    (" xrf_ui::padded =debug,warn", &["padded", "padded "]),
    ("warn,xrf_ui::views=info/text", &["views"]),
    ("warn,xrf_ui::a=debug/x/y", &["a"]),
    ("xrf_ui=error,,xrf_ui::a=debug", &["a"]),
    ("xrf_ui=error,xrf_ui::a", &["a"]),
    ("trace", &[]),
    ("warn", &[]),
    ("", &[]),
    (
      "info,xrf_ui::deep::scope=debug",
      &["deep", "deep::scope", "deep::scope::child"],
    ),
    ("xrf_ui=off,xrf_ui::on=info", &["on", "off"]),
    ("debug,xrf_ui=off", &["views"]),
  ];

  for (filters, scopes) in cases {
    let mut builder: Builder = Builder::new();

    LogFilter::from_directives(filters).configure(&mut builder);

    let logger: Logger = builder.build();
    let targets: Vec<String> = [String::from(UiLog::TARGET), format!("{}::unnamed", UiLog::TARGET)]
      .into_iter()
      .chain(scopes.iter().map(|scope| format!("{}::{scope}", UiLog::TARGET)))
      .collect();
    let written: Option<Level> = [Level::Debug, Level::Info, Level::Warn, Level::Error]
      .into_iter()
      .find(|level| {
        targets
          .iter()
          .any(|target| logger.enabled(&Metadata::builder().level(*level).target(target).build()))
      });

    assert_eq!(
      UiLogFilter::new(&LogFilter::from_directives(filters)).get_level(),
      written,
      "under '{filters}'"
    );
  }
}
