use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::core::directories::{get_executable_directory, get_local_data_directory};

/// This run's log file, opened fresh with the last run's kept beside it: under `appdata/logs` beside the executable, as
/// X-Ray keeps its own, else under the application's data directory.
pub struct LogFile {
  pub path: PathBuf,
  pub file: File,
}

impl LogFile {
  /// The log file's name, and the name the last run's is kept under.
  pub const NAME: &'static str = "xrf-app.log";
  pub const PREVIOUS_NAME: &'static str = "xrf-app.previous.log";

  /// Opens this run's log in the first directory that takes it, beside the executable or in the data directory of the
  /// bundle `identifier`.
  ///
  /// # Errors
  ///
  /// Returns why each directory refused it, where none did.
  pub fn open(identifier: &str) -> Result<Self, String> {
    let directories = [
      get_executable_directory().map(|directory| directory.join("appdata").join("logs")),
      get_local_data_directory(identifier).map(|directory| directory.join("logs")),
    ];
    let mut refusals: Vec<String> = Vec::new();

    for directory in directories.into_iter().flatten() {
      match Self::create_in(&directory) {
        Ok(log) => return Ok(log),
        Err(error) => refusals.push(format!("'{}': {error}", directory.display())),
      }
    }

    Err(refusals.join("; "))
  }

  /// Creates the log in `directory`, made where missing, moving a log already there aside as the previous one.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the directory or the file cannot be made.
  pub fn create_in(directory: &Path) -> std::io::Result<Self> {
    fs::create_dir_all(directory)?;

    let path: PathBuf = directory.join(Self::NAME);

    // A log that cannot be moved aside is overwritten: losing the last run's beats having no log for this one.
    if path.is_file() {
      let _ = fs::rename(&path, directory.join(Self::PREVIOUS_NAME));
    }

    File::create(&path).map(|file| Self { path, file })
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use super::LogFile;

  #[test]
  fn keeps_the_last_runs_log_beside_a_fresh_one() {
    let directory: PathBuf = std::env::temp_dir().join(format!("xrf-log-file-{}", std::process::id()));
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

    drop(second);
    let _ = fs::remove_dir_all(&directory);
  }
}
