use serde::{Deserialize, Serialize};

use crate::provenance::level_scene_instance_placement::LevelSceneInstancePlacement;
use crate::provenance::level_scene_library_object::LevelSceneLibraryObject;
use crate::provenance::level_scene_tree_match::LevelSceneTreeMatch;

/// The library objects a decompile looked for in the static geometry, and what it found: each object copied, with the
/// file it came from and the copy's hash, and each placement with how well it fits.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneInstances {
  /// The library roots looked in; a file there that could not be used is among the scene's refusals.
  pub roots: Vec<String>,
  pub objects: Vec<LevelSceneLibraryObject>,
  pub placements: Vec<LevelSceneInstancePlacement>,
  /// The tree models found to be library trees.
  #[serde(default)]
  pub trees: Vec<LevelSceneTreeMatch>,
}
