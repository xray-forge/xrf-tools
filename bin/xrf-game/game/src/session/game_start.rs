use xrf_game_alife::Alife;
use xrf_game_objects::ObjectRegistry;

use crate::level::game_player_config::GamePlayerConfig;

/// What a new game starts with, read from the tree's spawn and configs while the level opens: every object of the
/// spawn, A-Life over the game graph and the simulation's configs, and what the configs set for the player.
pub(crate) struct GameStart {
  pub(crate) objects: ObjectRegistry,
  pub(crate) alife: Alife,
  pub(crate) player: GamePlayerConfig,
}
