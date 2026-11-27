/// An ammo box's row in the ammo table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectAmmo {
  /// Rounds left. An empty box stays online (`CSE_ALifeItemAmmo::can_switch_offline`).
  pub rounds: u16,
}
