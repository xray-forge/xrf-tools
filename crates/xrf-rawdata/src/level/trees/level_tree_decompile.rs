use crate::level::geometry::level_decompiled_object::LevelDecompiledObject;
use crate::level::trees::level_tree_impostor::LevelTreeImpostor;
use crate::level::trees::level_tree_impostor_frame::LevelTreeImpostorFrame;
use crate::level::trees::level_tree_placement::LevelTreePlacement;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// A level's trees decompiled: an object per model, and each placement of one.
pub(crate) struct LevelTreeDecompile {
  /// The models, each at the origin of its own space.
  pub(crate) models: Vec<LevelDecompiledObject>,
  pub(crate) placements: Vec<LevelTreePlacement>,
  /// Each model's impostor strips, by its reference, where it has one.
  pub(crate) impostors: Vec<(String, LevelTreeImpostor)>,
  /// Each model given its strips, with where its impostor was stood, for
  /// [`LevelTreeConversion::fit_impostor_boxes`](crate::level::trees::level_tree_conversion::LevelTreeConversion::fit_impostor_boxes).
  pub(crate) frames: Vec<LevelTreeImpostorFrame>,
  /// The impostors that did not cut, the model coming back without one.
  pub(crate) refused: Vec<LevelSceneRefusal>,
}
