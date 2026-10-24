use serde::{Deserialize, Serialize};

/// What a level map's annotations draw, each kind on its own switch and in its own colour.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelAnnotationKind {
  /// The smart terrains' shapes, named.
  SmartTerrains,
  /// The space restrictors' shapes, those keeping the actor in apart from those keeping him out.
  Restrictors,
  /// The anomalous and torrid zones' shapes.
  Anomalies,
  /// The level changers' shapes, each with an arrow toward the level it leads to and its name.
  LevelChangers,
  /// The game graph's vertices on the level, as dots.
  GraphPoints,
  /// The patrol paths starting on the level: their links as lines, their points as dots, named at the first where
  /// every object is named.
  Patrols,
}

impl LevelAnnotationKind {
  /// Every kind, in the order they are drawn, the lowest first: the patrols' many lines under the names that matter.
  pub const ALL: [Self; 6] = [
    Self::GraphPoints,
    Self::Restrictors,
    Self::Anomalies,
    Self::Patrols,
    Self::SmartTerrains,
    Self::LevelChangers,
  ];

  /// Its group's id in the SVG.
  pub fn get_id(self) -> &'static str {
    match self {
      Self::SmartTerrains => "smart-terrains",
      Self::Restrictors => "restrictors",
      Self::Anomalies => "anomalies",
      Self::LevelChangers => "level-changers",
      Self::GraphPoints => "graph-points",
      Self::Patrols => "patrols",
    }
  }

  /// Its one colour: smart terrains orange, restrictors blue, anomalies red, level changers green, patrols yellow and
  /// graph points pale grey.
  pub fn get_colour(self) -> &'static str {
    match self {
      Self::SmartTerrains => "#ff9800",
      Self::Restrictors => "#2196f3",
      Self::Anomalies => "#f44336",
      Self::LevelChangers => "#4caf50",
      Self::GraphPoints => "#e0e0e0",
      Self::Patrols => "#ffeb3b",
    }
  }
}
