/// How many random members a squad draws (`npc_in_squad`): from `min` to `max`, both counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SquadSize {
  pub min: u32,
  pub max: u32,
}

impl Default for SquadSize {
  /// `sim_squad_scripted:create_npc`'s, for a squad that names none.
  fn default() -> Self {
    Self { min: 1, max: 2 }
  }
}
