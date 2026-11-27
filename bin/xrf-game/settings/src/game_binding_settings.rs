use serde::{Deserialize, Serialize};

/// The keys the player bound in place of the tree's own (`default_controls.ltx`), one per action, by the action's
/// X-Ray name (`forward`, `lstrafe`, `sprint_toggle`) and the key's SDL name (`W`, `Left Shift`); unset, the tree's
/// binding stands, so a mod's changed controls reach a player who never rebound that action. The settings file holds
/// only what the player bound.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBindingSettings {
  pub forward: Option<String>,
  pub back: Option<String>,
  pub lstrafe: Option<String>,
  pub rstrafe: Option<String>,
  pub jump: Option<String>,
  pub crouch: Option<String>,
  pub accel: Option<String>,
  pub sprint_toggle: Option<String>,
  /// Switching between walking and flight, the game's own action, which Call of Pripyat binds to nothing.
  pub noclip: Option<String>,
}
