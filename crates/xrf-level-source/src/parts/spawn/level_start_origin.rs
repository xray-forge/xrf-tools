/// What a level's start was taken from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelStartOrigin {
  /// Where a changer of another level leading here puts the actor.
  Arrival,
  /// Where the level's own actor is spawned.
  Actor,
}
