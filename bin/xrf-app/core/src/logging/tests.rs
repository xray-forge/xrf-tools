use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use xrf_test_utils::GeneratedResources;

use crate::logging::log_file::LogFile;
use crate::logging::log_run_name::LogRunName;

fn fresh(case: &str) -> PathBuf {
  let directory: PathBuf = GeneratedResources::path(&format!("log_file/{case}"));

  let _ = fs::remove_dir_all(&directory);

  directory
}

/// The name of a run that started `second` seconds into 2026 in the process `process`.
fn run(second: u64, process: u32) -> String {
  const NEW_YEAR_2026: u64 = 1_767_225_600;

  LogRunName::format(UNIX_EPOCH + Duration::from_secs(NEW_YEAR_2026 + second), process)
}

fn list_runs(directory: &PathBuf) -> Vec<String> {
  let mut runs: Vec<String> = fs::read_dir(directory)
    .expect("the logs list")
    .filter_map(Result::ok)
    .filter_map(|entry| entry.file_name().into_string().ok())
    .filter(|name| LogRunName::is_run(name))
    .collect();

  runs.sort();
  runs
}

#[test]
fn names_a_run_by_its_utc_second_and_its_process() {
  let leap_day: SystemTime = UNIX_EPOCH + Duration::from_secs(1_709_210_096);

  assert_eq!(
    LogRunName::format(leap_day, 4242),
    "xrf-app.2024-02-29T12-34-56Z.4242.log"
  );
  assert_eq!(LogRunName::format(UNIX_EPOCH, 7), "xrf-app.1970-01-01T00-00-00Z.7.log");
  assert_eq!(run(0, 1), "xrf-app.2026-01-01T00-00-00Z.1.log");
  assert_eq!(run(86_399, 1), "xrf-app.2026-01-01T23-59-59Z.1.log");
}

#[test]
fn tells_a_runs_log_from_the_single_logs_earlier_versions_wrote() {
  assert!(LogRunName::is_run("xrf-app.2026-11-22T10-30-05Z.4242.log"));
  assert!(!LogRunName::is_run("xrf-app.log"));
  assert!(!LogRunName::is_run("xrf-app.previous.log"));
  assert!(!LogRunName::is_run("xrf-app.2026-11-22T10-30-05Z.4242.txt"));
}

// Nothing keeps a second instance from starting: none may move or replace the log of one still writing it.
#[test]
fn keeps_a_running_instances_log_through_two_later_instances() {
  let directory: PathBuf = fresh("overlapping_instances");
  let mut first: LogFile = LogFile::create_in(&directory, &run(0, 1)).expect("the first log is made");

  first.file.write_all(b"first start\n").expect("the first writes");

  let second: LogFile = LogFile::create_in(&directory, &run(1, 2)).expect("the second log is made");
  let third: LogFile = LogFile::create_in(&directory, &run(2, 3)).expect("the third log is made");

  first.file.write_all(b"first later\n").expect("the first writes on");
  first.file.flush().expect("the first flushes");

  assert_eq!(
    fs::read_to_string(&first.path).expect("the first log reads"),
    "first start\nfirst later\n",
    "every line of the first run under its own name"
  );
  assert_ne!(first.path, second.path);
  assert_ne!(second.path, third.path);
  assert_eq!(list_runs(&directory).len(), 3);
}

#[test]
fn prunes_the_oldest_runs_beyond_what_it_keeps_and_leaves_the_single_logs() {
  let directory: PathBuf = fresh("pruned");

  fs::create_dir_all(&directory).expect("the logs folder is made");
  fs::write(directory.join("xrf-app.log"), "an earlier version's").expect("a single log");
  fs::write(directory.join("xrf-app.previous.log"), "an earlier version's").expect("a single log");

  for second in 0..LogFile::KEPT_RUNS as u64 + 2 {
    LogFile::create_in(&directory, &run(second, 1)).expect("a run's log is made");
  }

  let runs: Vec<String> = list_runs(&directory);

  assert_eq!(runs.len(), LogFile::KEPT_RUNS);
  assert_eq!(runs[0], run(2, 1), "the two oldest pruned");
  assert!(directory.join("xrf-app.log").is_file());
  assert!(directory.join("xrf-app.previous.log").is_file());
}

// The run that started first is the one most likely to have met the bug its log is kept for.
#[cfg(windows)]
#[test]
fn keeps_a_held_log_through_later_runs_pruning() {
  let directory: PathBuf = fresh("pruned_while_held");
  let held: LogFile = LogFile::create_in(&directory, &run(0, 1)).expect("the held log is made");

  for second in 1..LogFile::KEPT_RUNS as u64 + 2 {
    LogFile::create_in(&directory, &run(second, 2)).expect("a run's log is made");
  }

  assert!(held.path.is_file(), "the running instance's log stays");
  assert!(!directory.join(run(1, 2)).exists(), "an ended run's goes in its place");
}
