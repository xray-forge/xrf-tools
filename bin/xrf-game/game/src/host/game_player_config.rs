use xrf_game_movement::WalkMode;

/// What the tree's configs set for the player, read once as a new game starts: how they walk (`[actor]`) and how loud
/// their own steps play (`[hud_sound]`). What the player is next read from the configs is a field here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GamePlayerConfig {
  pub walk: WalkMode,
  /// How loud a first-person step plays against its power (`[hud_sound] hud_step_sound_vol_k`).
  pub step_volume: f32,
}
