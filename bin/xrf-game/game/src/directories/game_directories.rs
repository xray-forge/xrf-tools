use std::path::PathBuf;

use xrf_cache::AppCache;
use xrf_directories::DataDirectories;

/// Where the game writes: its own files under `appdata/xrf-game/` beside its executable when that directory accepts
/// files, else in its local data directory; its logs in the `appdata/logs/` it shares with the desktop app.
pub struct GameDirectories {
  data: DataDirectories,
}

impl GameDirectories {
  /// The game's bundle identifier, which names its local data directory, after the tools' `com.xrf.tools`.
  pub const IDENTIFIER: &'static str = "com.xrf.game";

  /// The folder within `appdata` the game keeps its own files in, apart from the desktop app's beside it.
  pub const FOLDER: &'static str = "xrf-game";

  /// The settings file within the game's own data.
  pub const SETTINGS_FILE: &'static str = "settings.ltx";

  /// The folder within the game's own data its caches are kept in, the renderer's pipeline cache among them.
  pub const CACHE_DIRECTORY: &'static str = "cache";

  /// The folder within the game's own data captures are written to.
  pub const CAPTURES_DIRECTORY: &'static str = "captures";

  /// The game's directories, its local data directory the system's own for its identifier.
  pub fn for_game() -> Self {
    Self {
      data: DataDirectories::for_identifier(Self::IDENTIFIER),
    }
  }

  /// Directories over `local_data`, as a test names them.
  pub fn new(local_data: Option<PathBuf>) -> Self {
    Self {
      data: DataDirectories::new(local_data),
    }
  }

  /// Where the game keeps its own files, best first: `appdata/xrf-game` beside the executable where that accepts files,
  /// then its local data directory.
  pub fn get_own_directories(&self) -> Vec<PathBuf> {
    Self::order_own_directories(
      DataDirectories::get_writable_executable_directory(),
      self.data.get_local_data().map(PathBuf::from),
    )
  }

  /// Where the game's logs are kept, best first, shared with the desktop app's.
  pub fn get_logs_directories(&self) -> Vec<PathBuf> {
    self.data.get_logs_directories()
  }

  /// The game's caches over the `cache` folder of its best own directory, and nothing else: a tidy removes files within
  /// it. None where the game has no own directory.
  pub fn get_cache(&self) -> Option<AppCache> {
    self
      .get_own_directories()
      .into_iter()
      .next()
      .map(|directory| AppCache::new(directory.join(Self::CACHE_DIRECTORY)))
  }

  /// Where captures the control endpoint asks for are written, in the game's best own directory; none where it has
  /// none.
  pub fn get_captures_directory(&self) -> Option<PathBuf> {
    self
      .get_own_directories()
      .into_iter()
      .next()
      .map(|directory| directory.join(Self::CAPTURES_DIRECTORY))
  }

  /// Where a game listening for a bridge writes how to attach to it: its best own directory, none where it has none.
  pub fn get_attach_directory(&self) -> Option<PathBuf> {
    self.get_own_directories().into_iter().next()
  }

  /// Where the settings file may be, best first: the first that exists is read.
  pub fn get_settings_files(&self) -> Vec<PathBuf> {
    self
      .get_own_directories()
      .into_iter()
      .map(|directory| directory.join(Self::SETTINGS_FILE))
      .collect()
  }

  pub(crate) fn order_own_directories(executable: Option<PathBuf>, local_data: Option<PathBuf>) -> Vec<PathBuf> {
    executable
      .map(|directory| directory.join(DataDirectories::APP_DATA_DIRECTORY).join(Self::FOLDER))
      .into_iter()
      .chain(local_data)
      .collect()
  }
}
