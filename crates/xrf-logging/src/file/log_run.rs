use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, io};

use xrf_error::{XrfError, XrfResult};

use crate::file::log_file::LogFile;
use crate::file::log_run_name::LogRunName;

/// This run's log, named for the run ([`LogRunName`]), made under the first of an application's logs directories that
/// takes it, with the last runs kept beside it; a run is kept or pruned whole, its companion files with its log.
pub struct LogRun {
  pub log: LogFile,
  name: LogRunName,
  directory: PathBuf,
}

impl LogRun {
  /// How many runs a directory keeps, this run's included.
  pub const KEPT_RUNS: usize = 10;

  /// Opens this run's log for `application` in the first of `directories` that takes it, made where missing.
  ///
  /// # Errors
  ///
  /// Returns why each directory refused the log, where none took it.
  pub fn open(application: &'static str, directories: &[PathBuf]) -> XrfResult<Self> {
    let name: LogRunName = LogRunName::new(application, SystemTime::now(), std::process::id());
    let mut refusals: Vec<String> = Vec::new();

    for directory in directories {
      match Self::create_in(directory, name) {
        Ok(run) => return Ok(run),
        Err(error) => refusals.push(format!("'{}': {error}", directory.display())),
      }
    }

    Err(XrfError::new_io_error(refusals.join("; "), ErrorKind::Other))
  }

  /// Creates a companion file of the run beside its log, named for the run ending `suffix`, such as `.ui.log`, which
  /// the run's pruning takes with its log.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the file cannot be made.
  pub fn create_companion(&self, suffix: &str) -> io::Result<LogFile> {
    LogFile::create(self.directory.join(self.name.format_companion(suffix)))
  }

  /// Creates the log of the run `name` in `directory`, made where missing, after pruning the oldest runs beyond what it
  /// keeps.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the directory or the log cannot be made.
  pub(crate) fn create_in(directory: &Path, name: LogRunName) -> io::Result<Self> {
    fs::create_dir_all(directory)?;
    Self::prune(directory, name);

    Ok(Self {
      log: LogFile::create(directory.join(name.format_log()))?,
      name,
      directory: directory.to_path_buf(),
    })
  }

  /// Removes the oldest runs of the application `about` names so that, with the run about to be made, at most
  /// [`Self::KEPT_RUNS`] remain, each run's log and companions together; a run is counted by its log alone. A file that
  /// will not go, such as one a running instance holds, stays: the next run tries again.
  fn prune(directory: &Path, about: LogRunName) {
    let Ok(entries) = fs::read_dir(directory) else {
      return;
    };
    let names: Vec<String> = entries
      .filter_map(Result::ok)
      .filter_map(|entry| entry.file_name().into_string().ok())
      .collect();
    let mut runs: Vec<LogRunName> = names
      .iter()
      .filter_map(|name| LogRunName::parse_log(about.get_application(), name))
      .collect();

    runs.sort();

    let excess: usize = runs.len().saturating_sub(Self::KEPT_RUNS - 1);

    for run in runs.into_iter().take(excess) {
      for name in names.iter().filter(|name| run.is_own_file(name)) {
        let _ = fs::remove_file(directory.join(name));
      }
    }
  }
}
