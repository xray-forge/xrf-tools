use xrf_game_alife::Alife;
use xrf_game_objects::ObjectRegistry;

/// What a new game starts with beside the player, read from the tree's spawn and configs while the level opens: every
/// object of the spawn, A-Life over the game graph and the simulation's configs, and how loud the player's own steps
/// play.
pub(crate) struct GameStart {
  pub(crate) objects: ObjectRegistry,
  pub(crate) alife: Alife,
  /// How loud a first-person step plays against its power (`[hud_sound] hud_step_sound_vol_k`).
  pub(crate) step_volume: f32,
}
