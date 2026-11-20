use serde::{Deserialize, Serialize};

/// An object the root spawn and `level.spawn` disagree on, past the fields xrAI fills in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneDisagreement {
  pub object: String,
  /// What differs: only one of the two holds it, or their spawn packets differ.
  pub reason: String,
}
