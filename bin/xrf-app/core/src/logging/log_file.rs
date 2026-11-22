use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::directories::AppDirectories;
use crate::logging::log_run_name::LogRunName;

/// This run's log file, of a name no other run takes ([`LogRunName`]), with the last runs' kept beside it: under `logs`
/// in the application's own data directory, `appdata` beside the executable as X-Ray keeps its own, else its local data
/// directory.
///
/// Instances run side by side, nothing keeping a second from starting, so no run moves or replaces another's log: each
/// writes its own, and a run prunes only the oldest beyond [`Self::KEPT_RUNS`]. On Windows a log is held without delete
/// sharing, so a run still writing it keeps it through any later run's pruning.
pub(super) struct LogFile {
  pub path: PathBuf,
  pub file: File,
}

impl LogFile {
  /// The directory the logs keep within the application's own data.
  pub(super) const DIRECTORY: &'static str = AppDirectories::LOGS_DIRECTORY;

  /// How many runs' logs a directory keeps, this run's included.
  pub(super) const KEPT_RUNS: usize = 10;

  /// `FILE_SHARE_READ | FILE_SHARE_WRITE`: another process reads or appends, none deletes or renames.
  #[cfg(windows)]
  const SHARE_READ_WRITE: u32 = 0x1 | 0x2;

  /// Opens this run's log in the first of the application's own data directories that takes it, the local one that of
  /// the bundle `identifier`.
  ///
  /// # Errors
  ///
  /// Returns why each directory refused it, where none did.
  pub(super) fn open(identifier: &str) -> Result<Self, String> {
    let mut refusals: Vec<String> = Vec::new();

    for directory in AppDirectories::for_identifier(identifier)
      .get_app_data_directories()
      .into_iter()
      .map(|directory| directory.join(Self::DIRECTORY))
    {
      match Self::create_in(&directory, &LogRunName::format(SystemTime::now(), std::process::id())) {
        Ok(log) => return Ok(log),
        Err(error) => refusals.push(format!("'{}': {error}", directory.display())),
      }
    }

    Err(refusals.join("; "))
  }

  /// Creates the log `name` in `directory`, made where missing, after pruning the oldest runs' beyond what it keeps.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the directory or the file cannot be made.
  pub(super) fn create_in(directory: &Path, name: &str) -> std::io::Result<Self> {
    fs::create_dir_all(directory)?;
    Self::prune(directory);

    let path: PathBuf = directory.join(name);
    let mut options: OpenOptions = OpenOptions::new();

    options.write(true).create(true).truncate(true);

    #[cfg(windows)]
    std::os::windows::fs::OpenOptionsExt::share_mode(&mut options, Self::SHARE_READ_WRITE);

    options.open(&path).map(|file| Self { path, file })
  }

  /// Removes the oldest runs' logs beyond the [`Self::KEPT_RUNS`] less one a new run leaves room for. A log that does not
  /// go, one a running instance holds, is kept: the next run tries again.
  fn prune(directory: &Path) {
    let Ok(entries) = fs::read_dir(directory) else {
      return;
    };
    let mut runs: Vec<PathBuf> = entries
      .filter_map(Result::ok)
      .filter(|entry| entry.file_name().to_str().is_some_and(LogRunName::is_run))
      .map(|entry| entry.path())
      .collect();

    runs.sort();

    let excess: usize = runs.len().saturating_sub(Self::KEPT_RUNS - 1);

    for run in runs.into_iter().take(excess) {
      let _ = fs::remove_file(run);
    }
  }
}
