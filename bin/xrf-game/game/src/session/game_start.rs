use xrf_game_alife::Alife;
use xrf_game_hooks::HookRegistry;
use xrf_game_objects::{ObjectId, ObjectRegistry};

use crate::host::game_configs::GameConfigs;
use crate::host::game_player_config::GamePlayerConfig;

/// What a new game starts with, read from the tree's spawn and configs while the level opens: every object of the
/// spawn and which is the player, A-Life over the game graph and the simulation's configs, the hook registry its
/// systems filled with their defaults, the graph level the game starts on, what the configs set for the player, and the
/// configs, which the run keeps.
pub(crate) struct GameStart {
  pub(crate) objects: ObjectRegistry,
  pub(crate) player: ObjectId,
  pub(crate) alife: Alife,
  pub(crate) hooks: HookRegistry,
  pub(crate) graph_level: Option<u8>,
  pub(crate) player_config: GamePlayerConfig,
  pub(crate) configs: GameConfigs,
}
