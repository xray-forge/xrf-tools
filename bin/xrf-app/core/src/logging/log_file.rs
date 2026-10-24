use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::directories::{get_app_data_directories, get_local_data_directory};

/// This run's log file, opened fresh with the last run's kept beside it: under `logs` in the application's own data
/// directory, `appdata` beside the executable as X-Ray keeps its own, else its local data directory.
pub struct LogFile {
  pub path: PathBuf,
  pub file: File,
}

impl LogFile {
  /// The directory the logs keep within the application's own data.
  pub const DIRECTORY: &'static str = "logs";

  /// The log file's name, and the name the last run's is kept under.
  pub const NAME: &'static str = "xrf-app.log";
  pub const PREVIOUS_NAME: &'static str = "xrf-app.previous.log";

  /// Opens this run's log in the first of the application's own data directories that takes it, the local one that of
  /// the bundle `identifier`.
  ///
  /// # Errors
  ///
  /// Returns why each directory refused it, where none did.
  pub fn open(identifier: &str) -> Result<Self, String> {
    let mut refusals: Vec<String> = Vec::new();

    for directory in get_app_data_directories(get_local_data_directory(identifier))
      .into_iter()
      .map(|directory| directory.join(Self::DIRECTORY))
    {
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
