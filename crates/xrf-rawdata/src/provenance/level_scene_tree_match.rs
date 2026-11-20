use serde::{Deserialize, Serialize};

/// A tree model a decompile found to be a library tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneTreeMatch {
  /// The model it stands for, by the reference the decompile would have given it, and the library tree.
  pub model: String,
  pub reference: String,
  /// How much of the tree's rendered area the model holds, and how far its corners stood from the model's, metres.
  pub coverage: f64,
  pub error: f64,
}
