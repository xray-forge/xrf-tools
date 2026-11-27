use xrf_error::XrfResult;
use xrf_game_developer::DeveloperPicture;

use crate::session::game_session::GameSession;

/// What a game opening tells the main thread, in this order: its level's picture once the tree is mounted, for the
/// loading view, where the tree has one; then the session, or why it did not open.
pub(crate) enum GameOpened {
  Picture(DeveloperPicture),
  Session(Box<XrfResult<GameSession>>),
}
