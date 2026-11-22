use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

use crate::directories::AppDirectories;
use crate::logging::file::LogFile;
use crate::logging::file::log_run_name::LogRunName;
use crate::tauri_result::TauriResult;

/// This run's pair of files, named for the run ([`LogRunName`]): its log, and its UI log beside it, which may fail to
/// open where the log opened. They are made under `logs` in the application's own data directory, `appdata` beside the
/// executable as X-Ray keeps its own, else its local data directory, with the last runs' kept beside them; a run is
/// kept or pruned whole.
pub(crate) struct LogRun {
  pub log: LogFile,
  pub ui_log: io::Result<LogFile>,
}

impl LogRun {
  /// How many runs a directory keeps, this run's included.
  pub(super) const KEPT_RUNS: usize = 10;

  /// Opens this run's files in the first of the application's own data directories that takes its log, the local one
  /// that of the bundle `identifier`.
  ///
  /// # Errors
  ///
  /// Returns why each directory refused the log, where none took it.
  pub(crate) fn open(identifier: &str) -> TauriResult<Self> {
    let name: LogRunName = LogRunName::new(SystemTime::now(), std::process::id());
    let mut refusals: Vec<String> = Vec::new();

    for directory in AppDirectories::for_identifier(identifier)
      .get_app_data_directories()
      .into_iter()
      .map(|directory| directory.join(AppDirectories::LOGS_DIRECTORY))
    {
      match Self::create_in(&directory, &name) {
        Ok(run) => return Ok(run),
        Err(error) => refusals.push(format!("'{}': {error}", directory.display())),
      }
    }

    Err(refusals.join("; "))
  }

  /// Creates the files of the run `name` in `directory`, made where missing, after pruning the oldest runs beyond what
  /// it keeps; the UI log is made only beside a log that was.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the directory or the log cannot be made.
  pub(super) fn create_in(directory: &Path, name: &LogRunName) -> io::Result<Self> {
    fs::create_dir_all(directory)?;
    Self::prune(directory);

    let log: LogFile = LogFile::create(directory.join(name.format_log()))?;
    let ui_log: io::Result<LogFile> = LogFile::create(directory.join(name.format_ui_log()));

    Ok(Self { log, ui_log })
  }

  /// Removes the oldest runs so that, with the run about to be made, at most [`Self::KEPT_RUNS`] remain, each run's log
  /// and UI log together; a run is counted by its log alone. A file that will not go, such as one a running instance
  /// holds, stays: the next run tries again.
  fn prune(directory: &Path) {
    let Ok(entries) = fs::read_dir(directory) else {
      return;
    };
    let mut runs: Vec<LogRunName> = entries
      .filter_map(Result::ok)
      .filter_map(|entry| entry.file_name().into_string().ok())
      .filter_map(|name| LogRunName::parse_log(&name))
      .collect();

    runs.sort();

    let excess: usize = runs.len().saturating_sub(Self::KEPT_RUNS - 1);

    for run in runs.into_iter().take(excess) {
      let _ = fs::remove_file(directory.join(run.format_ui_log()));
      let _ = fs::remove_file(directory.join(run.format_log()));
    }
  }
}
