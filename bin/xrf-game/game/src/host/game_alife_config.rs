use xrf_game_alife::AlifeSettings;

use crate::time::game_clock::GameClock;

/// What the tree's `[alife]` sets: A-Life's own settings, and the game's clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GameAlifeConfig {
  pub(crate) settings: AlifeSettings,
  pub(crate) clock: GameClock,
}
