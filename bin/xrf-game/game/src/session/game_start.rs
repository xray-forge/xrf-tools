use xrf_game_alife::Alife;
use xrf_game_objects::ObjectRegistry;

/// What a new game starts with beside the player, read from the tree's spawn and configs while the level opens: every
/// object of the spawn, and A-Life over the game graph and the simulation's configs.
pub(crate) struct GameStart {
  pub(crate) objects: ObjectRegistry,
  pub(crate) alife: Alife,
}
