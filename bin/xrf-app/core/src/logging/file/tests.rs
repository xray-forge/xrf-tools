use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use xrf_test_utils::GeneratedResources;

use crate::logging::file::log_run_name::LogRunName;
use crate::logging::file::{LogFile, LogRun};

const NEW_YEAR_2026: u64 = 1_767_225_600;

fn fresh(case: &str) -> PathBuf {
  let directory: PathBuf = GeneratedResources::path(&format!("log_file/{case}"));

  let _ = fs::remove_dir_all(&directory);

  directory
}

fn run(second: u64, process: u32) -> LogRunName {
  LogRunName::new(UNIX_EPOCH + Duration::from_secs(NEW_YEAR_2026 + second), process)
}

fn create_run(directory: &Path, name: &LogRunName) -> LogRun {
  LogRun::create_in(directory, name).expect("a run's log is made")
}

fn list_runs(directory: &Path) -> Vec<LogRunName> {
  let mut runs: Vec<LogRunName> = list_names(directory)
    .iter()
    .filter_map(|name| LogRunName::parse_log(name))
    .collect();

  runs.sort();
  runs
}

fn list_names(directory: &Path) -> Vec<String> {
  let mut names: Vec<String> = fs::read_dir(directory)
    .expect("the logs list")
    .filter_map(Result::ok)
    .filter_map(|entry| entry.file_name().into_string().ok())
    .collect();

  names.sort();
  names
}

#[test]
fn names_a_run_by_its_utc_second_and_its_process() {
  let leap_day: SystemTime = UNIX_EPOCH + Duration::from_secs(1_709_210_096);

  assert_eq!(
    LogRunName::new(leap_day, 4242).format_log(),
    "xrf-app.2024-02-29T12-34-56Z.4242.log"
  );
  assert_eq!(
    LogRunName::new(leap_day + Duration::from_millis(999), 4242),
    LogRunName::new(leap_day, 4242),
    "a run is named to its second"
  );
  assert_eq!(
    LogRunName::new(UNIX_EPOCH, 7).format_log(),
    "xrf-app.1970-01-01T00-00-00Z.7.log"
  );
  assert_eq!(run(0, 1).format_log(), "xrf-app.2026-01-01T00-00-00Z.1.log");
  assert_eq!(run(86_399, 1).format_log(), "xrf-app.2026-01-01T23-59-59Z.1.log");
  assert_eq!(run(0, 1).format_ui_log(), "xrf-app.2026-01-01T00-00-00Z.1.ui.log");
}

#[test]
fn reads_a_run_back_from_its_logs_name() {
  for name in [run(0, 1), run(86_399, 4242), run(5, u32::MAX)] {
    assert_eq!(LogRunName::parse_log(&name.format_log()), Some(name));
  }

  // 2026-11-22, 325 days into the year, at 10:30:05.
  assert_eq!(
    LogRunName::parse_log("xrf-app.2026-11-22T10-30-05Z.4242.log"),
    Some(run(325 * 86_400 + 37_805, 4242))
  );
}

#[test]
fn tells_a_runs_log_from_its_ui_log_and_the_single_logs_earlier_versions_wrote() {
  assert!(LogRunName::parse_log("xrf-app.2026-11-22T10-30-05Z.4242.log").is_some());
  assert!(LogRunName::parse_log("xrf-app.2026-11-22T10-30-05Z.4242.ui.log").is_none());
  assert!(LogRunName::parse_log("xrf-app.log").is_none());
  assert!(LogRunName::parse_log("xrf-app.previous.log").is_none());
  assert!(LogRunName::parse_log("xrf-app.2026-11-22T10-30-05Z.4242.txt").is_none());
}

// Pruning removes files, so a name no run of this application would write is never taken for a run's.
#[test]
fn takes_no_name_a_run_would_not_write_for_a_runs() {
  for name in [
    "xrf-app.2026-13-22T10-30-05Z.4242.log",
    "xrf-app.2026-11-22T10-30-05Z.+4242.log",
    "xrf-app.2026-11-22T10-30-05Z.04242.log",
    "xrf-app.2026-11-22T10-30-05Z..log",
    "xrf-app.2026-11-22T10:30:05Z.4242.log",
    "xrf-app.2026-11-22.4242.log",
    "xrf-app.2026-11-22T10-30-05Z.99999999999.log",
  ] {
    assert_eq!(LogRunName::parse_log(name), None, "'{name}'");
  }
}

// Nothing keeps a second instance from starting: none may move or replace the log of one still writing it.
#[test]
fn keeps_a_running_instances_log_through_two_later_instances() {
  let directory: PathBuf = fresh("overlapping_instances");
  let mut first: LogRun = create_run(&directory, &run(0, 1));

  first.log.file.write_all(b"first start\n").expect("the first writes");

  let second: LogRun = create_run(&directory, &run(1, 2));
  let third: LogRun = create_run(&directory, &run(2, 3));

  first.log.file.write_all(b"first later\n").expect("the first writes on");
  first.log.file.flush().expect("the first flushes");

  assert_eq!(
    fs::read_to_string(&first.log.path).expect("the first log reads"),
    "first start\nfirst later\n",
    "every line of the first run under its own name"
  );
  assert_ne!(first.log.path, second.log.path);
  assert_ne!(second.log.path, third.log.path);
  assert_eq!(list_runs(&directory).len(), 3);
}

#[test]
fn prunes_the_oldest_runs_beyond_what_it_keeps_and_leaves_the_single_logs() {
  let directory: PathBuf = fresh("pruned");

  fs::create_dir_all(&directory).expect("the logs folder is made");
  fs::write(directory.join("xrf-app.log"), "an earlier version's").expect("a single log");
  fs::write(directory.join("xrf-app.previous.log"), "an earlier version's").expect("a single log");

  for second in 0..LogRun::KEPT_RUNS as u64 + 2 {
    create_run(&directory, &run(second, 1));
  }

  let runs: Vec<LogRunName> = list_runs(&directory);

  assert_eq!(runs.len(), LogRun::KEPT_RUNS);
  assert_eq!(runs[0], run(2, 1), "the two oldest pruned");
  assert!(directory.join("xrf-app.log").is_file());
  assert!(directory.join("xrf-app.previous.log").is_file());
}

#[test]
fn makes_a_runs_ui_log_beside_its_log_under_the_runs_name() {
  let directory: PathBuf = fresh("ui_beside");
  let LogRun { log, ui_log } = create_run(&directory, &run(0, 1));
  let ui_log: LogFile = ui_log.expect("a run's UI log is made");

  assert_eq!(ui_log.path.parent(), log.path.parent());
  assert_eq!(
    list_names(&directory),
    [run(0, 1).format_log(), run(0, 1).format_ui_log()]
  );
}

#[test]
fn counts_a_run_and_its_ui_log_as_one_run() {
  let directory: PathBuf = fresh("ui_not_a_run");

  for second in 0..LogRun::KEPT_RUNS as u64 {
    create_run(&directory, &run(second, 1));
  }

  assert_eq!(
    list_runs(&directory).len(),
    LogRun::KEPT_RUNS,
    "no run pruned for its UI log"
  );
  assert_eq!(list_names(&directory).len(), 2 * LogRun::KEPT_RUNS);
}

#[test]
fn prunes_a_runs_ui_log_with_its_log() {
  let directory: PathBuf = fresh("ui_pruned");

  for second in 0..LogRun::KEPT_RUNS as u64 + 2 {
    create_run(&directory, &run(second, 1));
  }

  let names: Vec<String> = list_names(&directory);

  assert_eq!(names.len(), 2 * LogRun::KEPT_RUNS);

  for pruned in [run(0, 1), run(1, 1)] {
    assert!(!names.contains(&pruned.format_ui_log()), "the pruned run's UI log goes");
    assert!(!names.contains(&pruned.format_log()), "the pruned run's log goes");
  }

  assert!(
    names.contains(&run(2, 1).format_ui_log()),
    "a kept run keeps its UI log"
  );
}

// The run that started first is the one most likely to have met the bug its log is kept for.
#[cfg(windows)]
#[test]
fn keeps_a_held_log_through_later_runs_pruning() {
  let directory: PathBuf = fresh("pruned_while_held");
  let held: LogRun = create_run(&directory, &run(0, 1));

  for second in 1..LogRun::KEPT_RUNS as u64 + 2 {
    create_run(&directory, &run(second, 2));
  }

  assert!(held.log.path.is_file(), "the running instance's log stays");
  assert!(
    held
      .ui_log
      .expect("the running instance's UI log is made")
      .path
      .is_file(),
    "the running instance's UI log stays"
  );
  assert!(
    !directory.join(run(1, 2).format_log()).exists(),
    "an ended run's goes in its place"
  );
  assert!(
    !directory.join(run(1, 2).format_ui_log()).exists(),
    "and its UI log with it"
  );
}
