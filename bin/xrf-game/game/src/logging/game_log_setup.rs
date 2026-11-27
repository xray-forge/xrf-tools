use xrf_logging::{LogFilter, LogSetup};

use crate::directories::game_directories::GameDirectories;

/// How the game starts logging: the process's logger, writing to the console and to this run's
/// `xrf-game.<UTC start>.<process>.log` beside the desktop app's logs.
pub struct GameLogSetup;

impl GameLogSetup {
  /// What the game's log files are named by.
  pub const LOG_NAME: &'static str = "xrf-game";

  /// Installs the process's logger, filtered by `RUST_LOG` else the build's default level.
  ///
  /// # Panics
  ///
  /// Where the process already has a logger: call it once, before anything logs.
  pub fn install(directories: &GameDirectories, version: &str) {
    let setup: LogSetup = LogSetup {
      application: Self::LOG_NAME,
      version: String::from(version),
      directories: directories.get_logs_directories(),
    };

    // The logger holds its own handle to the log; the run itself only makes companion files, which the game has none of.
    let _ = setup.install(&LogFilter::from_environment());
  }
}
