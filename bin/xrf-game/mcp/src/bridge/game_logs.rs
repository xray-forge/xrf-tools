use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use xrf_directories::DataDirectories;
use xrf_error::{XrfError, XrfResult};

/// The game's log files as the bridge reads them from disk, so a game that hung or crashed still has its log read.
pub struct GameLogs {
  directories: Vec<PathBuf>,
}

impl GameLogs {
  /// The game's local data identifier, which names the folder its logs fall back to.
  const IDENTIFIER: &'static str = "com.xrf.game";

  /// What every game log's name starts with; the bridge's own never does.
  const PREFIX: &'static str = "xrf-game.";

  /// The game's log directories, found as the game finds them: beside the executable both sit beside, else its local
  /// data directory.
  pub fn locate() -> Self {
    Self {
      directories: DataDirectories::for_identifier(Self::IDENTIFIER).get_logs_directories(),
    }
  }

  /// Logs in `directories` alone, as a test names them.
  pub fn new(directories: Vec<PathBuf>) -> Self {
    Self { directories }
  }

  /// The last `lines` lines of the newest game log, with its path.
  ///
  /// # Errors
  ///
  /// Returns why no log can be read: the game never wrote one, or it cannot be read.
  pub fn read_tail(&self, lines: usize) -> XrfResult<String> {
    let path: PathBuf = self
      .find_newest()
      .ok_or_else(|| XrfError::new_not_found_error("the game has written no log yet"))?;
    let text: String = String::from_utf8_lossy(&fs::read(&path)?).into_owned();
    let all: Vec<&str> = text.lines().collect();
    let tail: &[&str] = &all[all.len().saturating_sub(lines)..];

    Ok(format!("{}\n{}", path.display(), tail.join("\n")))
  }

  fn find_newest(&self) -> Option<PathBuf> {
    self
      .directories
      .iter()
      .filter_map(|directory| fs::read_dir(directory).ok())
      .flatten()
      .filter_map(Result::ok)
      .map(|entry| entry.path())
      .filter(|path| Self::is_game_log(path))
      .max_by_key(|path| {
        fs::metadata(path)
          .and_then(|metadata| metadata.modified())
          .unwrap_or(SystemTime::UNIX_EPOCH)
      })
  }

  fn is_game_log(path: &Path) -> bool {
    let is_log: bool = path
      .extension()
      .is_some_and(|extension| extension.eq_ignore_ascii_case("log"));

    is_log
      && path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(Self::PREFIX))
  }
}
