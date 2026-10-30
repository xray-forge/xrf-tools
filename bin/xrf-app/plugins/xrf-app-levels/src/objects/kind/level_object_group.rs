use serde::Serialize;

/// The family a level object's kind belongs to, which the Outliner and the Show popover nest kinds under.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelObjectGroup {
  /// The spawn's objects drawn as a model.
  Spawned,
  /// Who the spawn places, drawn as nothing: the actor's start, stalkers, creatures, traders and squads.
  Characters,
  /// The logic volumes: smart terrains, restrictors, anomalies, level changers and ladders.
  Volumes,
  /// The smart covers.
  Covers,
}
