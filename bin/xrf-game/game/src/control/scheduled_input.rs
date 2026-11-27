use std::time::Instant;

use xrf_game_platform::PlatformEventKind;

/// An event an agent played, delivered to the game once `due` has come.
#[derive(Clone, Debug)]
pub(crate) struct ScheduledInput {
  pub due: Instant,
  pub kind: PlatformEventKind,
}
