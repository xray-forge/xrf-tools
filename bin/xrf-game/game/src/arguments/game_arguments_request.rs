use std::time::{SystemTime, UNIX_EPOCH};

use xrf_error::{XrfError, XrfResult};
use xrf_game_movement::MovementMode;

use crate::arguments::game_arguments::GameArguments;
use crate::arguments::game_mode_argument::GameModeArgument;
use crate::host::game_tree::GameTree;
use crate::record::game_recording_header::GameRecordingHeader;
use crate::record::game_replay::GameReplay;
use crate::session::game_load_request::GameLoadRequest;
use crate::session::game_load_source::GameLoadSource;
use crate::session::game_recording_request::GameRecordingRequest;

impl GameArguments {
  /// The load the command line asks for: its tree and level, a new game keyed by its seed or by one made now, how the
  /// player moves, and the recording the session is written to; or, for a replay, its tree and what the recording's
  /// header says the session opened.
  ///
  /// # Errors
  ///
  /// Returns why it asks for none: no tree named, no level, then with the levels the tree holds, or a recording to
  /// replay whose header does not read.
  pub fn read_request(&self) -> XrfResult<GameLoadRequest> {
    let Some(root) = &self.root else {
      return Err(XrfError::new_invalid_error(
        "name the game tree to play with --root <path> and its level with --level <name>",
      ));
    };

    // A replay opens what its recording did, whatever else the command line names.
    if let Some(file) = &self.replay {
      let header: GameRecordingHeader = GameReplay::read_header(file)?;

      return Ok(GameLoadRequest {
        tree: root.clone(),
        level: header.level,
        game: GameLoadSource::New { seed: header.seed },
        mode: header.mode,
        recording: Some(GameRecordingRequest::Replay(file.clone())),
      });
    }

    let Some(level) = self.level.clone() else {
      return Err(XrfError::new_invalid_error(format!(
        "name the level to play with --level <name>: the tree has {}",
        GameTree::mount(root)?.list_levels()?.join(", ")
      )));
    };

    Ok(GameLoadRequest {
      tree: root.clone(),
      level,
      game: GameLoadSource::New {
        seed: self.seed.unwrap_or_else(Self::create_seed),
      },
      mode: match self.mode {
        GameModeArgument::Fly => MovementMode::Fly,
        GameModeArgument::Walk => MovementMode::Walk,
      },
      recording: self.record.clone().map(GameRecordingRequest::Record),
    })
  }

  /// A seed for a run that names none: the wall clock's nanoseconds, the one place a new game reads it.
  fn create_seed() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0, |since| since.as_nanos() as u64)
  }
}
