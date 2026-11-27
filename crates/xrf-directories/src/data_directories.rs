use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, fs};

/// Where an application keeps its own files: `appdata` beside its executable when that directory accepts files, as a
/// portable install keeps everything together and X-Ray keeps its own `appdata`, else its own local data directory.
/// Each kind of place is a list of candidates, best first.
pub struct DataDirectories {
  /// Writable directory belonging to this application, used when its own is not writable.
  local_data: Option<PathBuf>,
}

impl DataDirectories {
  /// Directory beside the executable for what the application keeps of its own: logs, settings, caches, and files a
  /// tool keeps for its user.
  pub const APP_DATA_DIRECTORY: &'static str = "appdata";

  /// Directory within the application's own data the logs are kept in, which applications sharing an `appdata` share.
  pub const LOGS_DIRECTORY: &'static str = "logs";

  /// Directories over `local_data`, the application's local data directory where it has one.
  pub fn new(local_data: Option<PathBuf>) -> Self {
    Self { local_data }
  }

  /// Directories of the application whose bundle is `identifier`, such as `com.xrf.tools`, its local data directory
  /// the system's own for it, so they are known before anything else of the application runs.
  pub fn for_identifier(identifier: &str) -> Self {
    Self::new(dirs::data_local_dir().map(|directory| directory.join(identifier)))
  }

  /// Where the application keeps its own files, best first: `appdata` beside the executable where that accepts files,
  /// then its local data directory.
  pub fn get_app_data_directories(&self) -> Vec<PathBuf> {
    Self::order_app_data_directories(Self::get_writable_executable_directory(), self.local_data.clone())
  }

  /// Where the application's logs are kept, best first: `logs` in each of its own data directories.
  pub fn get_logs_directories(&self) -> Vec<PathBuf> {
    self
      .get_app_data_directories()
      .into_iter()
      .map(|directory| directory.join(Self::LOGS_DIRECTORY))
      .collect()
  }

  /// The application's local data directory, none where the system has none.
  pub fn get_local_data(&self) -> Option<&Path> {
    self.local_data.as_deref()
  }

  /// The directory the running executable lies in, where it accepts files: decided once a run, so every caller
  /// addresses the same folders. Probed on every ask, two at once could see each other's probe being removed, and one
  /// decide the other way.
  pub fn get_writable_executable_directory() -> Option<PathBuf> {
    static WRITABLE: OnceLock<Option<PathBuf>> = OnceLock::new();

    WRITABLE
      .get_or_init(|| {
        env::current_exe()
          .ok()
          .and_then(|executable| executable.parent().map(Path::to_path_buf))
          .filter(|directory| Self::is_writable(directory))
      })
      .clone()
  }

  pub(crate) fn order_app_data_directories(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> Vec<PathBuf> {
    executable
      .map(|directory| directory.join(Self::APP_DATA_DIRECTORY))
      .into_iter()
      .chain(local_data)
      .collect()
  }

  /// Whether a directory accepts a file, tested by writing one of this process's own and removing it again.
  fn is_writable(directory: &Path) -> bool {
    let probe: PathBuf = directory.join(format!(".xrf-write-probe-{}", std::process::id()));

    match fs::write(&probe, []) {
      Ok(()) => {
        let _ = fs::remove_file(&probe);

        true
      }
      Err(_) => false,
    }
  }
}
