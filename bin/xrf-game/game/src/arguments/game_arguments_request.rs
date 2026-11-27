use std::time::{SystemTime, UNIX_EPOCH};

use xrf_error::{XrfError, XrfResult};
use xrf_game_movement::MovementMode;

use crate::arguments::game_arguments::GameArguments;
use crate::arguments::game_mode_argument::GameModeArgument;
use crate::host::game_tree::GameTree;
use crate::session::game_load_request::GameLoadRequest;
use crate::session::game_load_source::GameLoadSource;

impl GameArguments {
  /// The load the command line asks for: its tree and level, a new game keyed by its seed or by one made now, and how
  /// the player moves.
  ///
  /// # Errors
  ///
  /// Returns why it asks for none: no tree named, or no level, then with the levels the tree holds.
  pub fn read_request(&self) -> XrfResult<GameLoadRequest> {
    let Some(root) = &self.root else {
      return Err(XrfError::new_invalid_error(
        "name the game tree to play with --root <path> and its level with --level <name>",
      ));
    };
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
    })
  }

  /// A seed for a run that names none: the wall clock's nanoseconds, the one place a new game reads it.
  fn create_seed() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0, |since| since.as_nanos() as u64)
  }
}
