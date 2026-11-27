/// A creature's row in the creatures table: what is alive about a stalker, a monster or the actor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectCreature {
  /// From 0, dead, to 1. A dead creature stays online (`CSE_ALifeCreatureAbstract::can_switch_offline`).
  pub health: f32,
}
