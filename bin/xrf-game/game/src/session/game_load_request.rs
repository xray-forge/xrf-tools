use std::path::PathBuf;

use xrf_game_movement::MovementMode;

use crate::session::game_load_source::GameLoadSource;
use crate::session::game_recording_request::GameRecordingRequest;

/// What a load is asked to open: the game tree, the level on it, the game played there, and how the player moves.
/// Whoever asks (the command line now, a menu later) builds one; the load reads nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameLoadRequest {
  /// The tree's root folder: an installation, unpacked `gamedata` or packed archives, as the tools mount it.
  pub tree: PathBuf,
  /// The level's name in the tree.
  pub level: String,
  pub game: GameLoadSource,
  pub mode: MovementMode,
  /// The recording the session is written to or stepped from; none for a session played and not kept.
  pub recording: Option<GameRecordingRequest>,
}
