use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, fs};

/// Where this application writes: beside its executable when that directory accepts files, as a portable install keeps
/// everything together, else in its own local data directory. Each kind of place is a list of candidates, best first.
pub struct AppDirectories {
  /// Writable directory belonging to this application, used when its own is not writable.
  local_data: Option<PathBuf>,
}

impl AppDirectories {
  /// Directory beside the executable for what the application keeps of its own, as X-Ray keeps its `appdata`: logs,
  /// preferences, caches, and files a tool keeps for its user.
  pub const APP_DATA_DIRECTORY: &'static str = "appdata";

  /// Directory, beside the executable or in the local data directory, for everything tools write.
  pub const OUTPUT_DIRECTORY: &'static str = "target";

  /// Directory within the application's own data that caches rebuilt from nothing are kept in.
  pub const CACHE_DIRECTORY: &'static str = "cache";

  /// File within the application's own data the backend keeps its preferences in.
  pub const PREFERENCES_FILE: &'static str = "preferences.json";

  /// Directories over `local_data`, the application's local data directory where it has one.
  pub fn new(local_data: Option<PathBuf>) -> Self {
    Self { local_data }
  }

  /// Directories of the application whose bundle is `identifier`, its local data directory resolved as Tauri resolves
  /// it, so they are known before Tauri runs.
  pub fn for_identifier(identifier: &str) -> Self {
    Self::new(dirs::data_local_dir().map(|directory| directory.join(identifier)))
  }

  /// Where the application keeps its own files, best first: `appdata` beside the executable where that accepts files,
  /// then its local data directory.
  pub fn get_app_data_directories(&self) -> Vec<PathBuf> {
    Self::order_app_data_directories(Self::get_writable_executable_directory(), self.local_data.clone())
  }

  /// Where tools write when no output directory is configured, best first: `target` beside the executable where that
  /// accepts files, then in the local data directory.
  pub fn get_output_directories(&self) -> Vec<PathBuf> {
    Self::order_output_directories(Self::get_writable_executable_directory(), self.local_data.clone())
  }

  /// The application's own data directory as the page addresses it, the first of them with a Unicode path.
  pub fn get_app_data_root(&self) -> Option<String> {
    Self::find_first_unicode_path(self.get_app_data_directories())
  }

  /// The directory tools write to by default as the page addresses it, the first of them with a Unicode path.
  pub fn get_output_root(&self) -> Option<String> {
    Self::find_first_unicode_path(self.get_output_directories())
  }

  /// The file the backend's preferences are kept in, in the best of the application's own data directories.
  pub fn get_preferences_file(&self) -> Option<PathBuf> {
    self
      .get_app_data_directories()
      .into_iter()
      .next()
      .map(|directory| directory.join(Self::PREFERENCES_FILE))
  }

  /// The directory the cache `name` is kept in, in the best of the application's own data directories.
  pub fn get_cache_directory(&self, name: &str) -> Option<PathBuf> {
    self
      .get_app_data_directories()
      .into_iter()
      .next()
      .map(|directory| directory.join(Self::CACHE_DIRECTORY).join(name))
  }

  /// The first of `directories` the page can address without changing it: one with a Unicode path.
  pub(super) fn find_first_unicode_path(directories: Vec<PathBuf>) -> Option<String> {
    directories
      .into_iter()
      .find_map(|directory| directory.into_os_string().into_string().ok())
  }

  pub(super) fn order_app_data_directories(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> Vec<PathBuf> {
    executable
      .map(|directory| directory.join(Self::APP_DATA_DIRECTORY))
      .into_iter()
      .chain(local_data)
      .collect()
  }

  pub(super) fn order_output_directories(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> Vec<PathBuf> {
    executable
      .into_iter()
      .chain(local_data)
      .map(|directory| directory.join(Self::OUTPUT_DIRECTORY))
      .collect()
  }

  /// The directory the running executable lies in, where it accepts files: decided once a run, so every command
  /// addresses the same folders. Probed on every ask, two at once could see each other's probe being removed, and one
  /// decide the other way.
  fn get_writable_executable_directory() -> Option<PathBuf> {
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
