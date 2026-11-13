use std::fmt;

/// Why the spawn build leaves a graph point out of the game graph, as xrAI does with a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnGraphPointRemoval {
  /// Within a millimetre of a graph point kept before it (`distance_to_sqr < EPS_L`).
  TooClose,
  /// Over no node of the AI map.
  OutsideMap,
  /// Over the node of a graph point kept before it.
  SharedNode,
}

impl fmt::Display for SpawnGraphPointRemoval {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::TooClose => "too close to another graph point",
      Self::OutsideMap => "outside the AI map",
      Self::SharedNode => "on the same AI node as another graph point",
    })
  }
}
