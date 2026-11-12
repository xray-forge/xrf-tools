use std::path::PathBuf;

/// A library tree a decompile found a level's tree model to be, which the scene places by reference instead of the
/// model.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelTreeMatch {
  /// The model it stands for, by the reference the decompile would have given it.
  pub model: String,
  /// The library tree, by its reference, and the file it was read from.
  pub reference: String,
  pub path: PathBuf,
  /// How much of the tree's rendered area the model holds, and how far its corners stood from the model's, metres.
  pub coverage: f64,
  pub error: f64,
}
