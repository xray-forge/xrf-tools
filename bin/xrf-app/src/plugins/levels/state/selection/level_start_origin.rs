use serde::Serialize;

/// What a level's start was taken from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelStartOrigin {
  /// Where a changer of another level leading here puts the actor.
  Arrival,
  /// Where the level's own actor is spawned.
  Actor,
}
