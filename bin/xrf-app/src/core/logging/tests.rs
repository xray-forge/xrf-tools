use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::core::logging::log_file::LogFile;

#[test]
fn keeps_the_last_runs_log_beside_a_fresh_one() {
  let directory: PathBuf = build_absolute_generated_test_resource_path("log_file/keeps_the_last_runs_log");

  let _ = fs::remove_dir_all(&directory);

  LogFile::create_in(&directory).expect("the first log is made");
  fs::write(directory.join(LogFile::NAME), "first run").expect("the first run writes its log");

  let second: LogFile = LogFile::create_in(&directory).expect("the second log is made");

  assert_eq!(second.path, directory.join(LogFile::NAME));
  assert_eq!(fs::read_to_string(&second.path).expect("the new log reads"), "");
  assert_eq!(
    fs::read_to_string(directory.join(LogFile::PREVIOUS_NAME)).expect("the previous log reads"),
    "first run"
  );
}
