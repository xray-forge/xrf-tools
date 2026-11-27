use std::path::PathBuf;

use xrf_cache::AppCache;
use xrf_directories::DataDirectories;

/// Where this application writes: its own data where `xrf-directories` places an application's (`appdata` beside its
/// executable when that directory accepts files, else its local data directory), and what it keeps there. Each kind of
/// place is a list of candidates, best first.
pub struct AppDirectories {
  data: DataDirectories,
}

impl AppDirectories {
  /// Directory beside the executable for what the application keeps of its own, as X-Ray keeps its `appdata`: logs,
  /// preferences, caches, and files a tool keeps for its user.
  pub const APP_DATA_DIRECTORY: &'static str = DataDirectories::APP_DATA_DIRECTORY;

  /// Directory, beside the executable or in the local data directory, for everything tools write.
  pub const OUTPUT_DIRECTORY: &'static str = "target";

  /// Directory within the application's own data that caches rebuilt from nothing are kept in.
  pub const CACHE_DIRECTORY: &'static str = "cache";

  /// Directory within the application's own data the logs are kept in.
  pub const LOGS_DIRECTORY: &'static str = DataDirectories::LOGS_DIRECTORY;

  /// File within the application's own data the backend keeps its preferences in.
  pub const PREFERENCES_FILE: &'static str = "preferences.json";

  /// Directories over `local_data`, the application's local data directory where it has one.
  pub fn new(local_data: Option<PathBuf>) -> Self {
    Self {
      data: DataDirectories::new(local_data),
    }
  }

  /// Directories of the application whose bundle is `identifier`, its local data directory resolved as Tauri resolves
  /// it, so they are known before Tauri runs.
  pub fn for_identifier(identifier: &str) -> Self {
    Self {
      data: DataDirectories::for_identifier(identifier),
    }
  }

  /// Where the application keeps its own files, best first: `appdata` beside the executable where that accepts files,
  /// then its local data directory.
  pub fn get_app_data_directories(&self) -> Vec<PathBuf> {
    self.data.get_app_data_directories()
  }

  /// Where the application's logs are kept, best first.
  pub fn get_logs_directories(&self) -> Vec<PathBuf> {
    self.data.get_logs_directories()
  }

  /// Where tools write when no output directory is configured, best first: `target` beside the executable where that
  /// accepts files, then in the local data directory.
  pub fn get_output_directories(&self) -> Vec<PathBuf> {
    Self::order_output_directories(
      DataDirectories::get_writable_executable_directory(),
      self.data.get_local_data().map(PathBuf::from),
    )
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

  /// The directory the caches are kept in (`AppCache`), in the best of the application's own data directories.
  pub fn get_cache_root(&self) -> Option<PathBuf> {
    self
      .get_app_data_directories()
      .into_iter()
      .next()
      .map(|directory| directory.join(Self::CACHE_DIRECTORY))
  }

  /// The application's caches, over its cache folder alone, none where it has no data directory: a tidy removes files
  /// within that folder, and never reaches the tools' saved files, logs or exports beside it.
  pub fn get_cache(&self) -> Option<AppCache> {
    self.get_cache_root().map(AppCache::new)
  }

  /// The first of `directories` the page can address without changing it: one with a Unicode path.
  pub(super) fn find_first_unicode_path(directories: Vec<PathBuf>) -> Option<String> {
    directories
      .into_iter()
      .find_map(|directory| directory.into_os_string().into_string().ok())
  }

  pub(super) fn order_output_directories(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> Vec<PathBuf> {
    executable
      .into_iter()
      .chain(local_data)
      .map(|directory| directory.join(Self::OUTPUT_DIRECTORY))
      .collect()
  }
}
