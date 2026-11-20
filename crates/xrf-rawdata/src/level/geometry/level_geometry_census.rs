use serde::{Deserialize, Serialize};

/// What the static geometry's decompile counts, so every triangle the compiled level draws is either in a written
/// object or accounted for.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeometryCensus {
  /// Full-detail triangles of the sectors' static visuals, read.
  pub static_triangles: usize,
  /// Faces the written objects hold.
  pub faces: usize,
  /// Faces of those that are two-sided, each folded from a triangle and its mirror.
  pub two_sided_faces: usize,
  /// Triangles dropped as an exact repeat of another of their surface.
  pub repeated_triangles: usize,
  /// Full-detail triangles of tree instances, every placement's counted; each model holds one placement's.
  pub tree_triangles: usize,
  /// Tree models found to be library trees, which their placements reference instead.
  #[serde(default)]
  pub library_tree_models: usize,
  /// Multi-use models the trees come back as, and their placements.
  #[serde(default)]
  pub tree_models: usize,
  #[serde(default)]
  pub tree_placements: usize,
  /// Library objects found placed in the static geometry, the distinct objects they place, and the static triangles
  /// their placements took out of the shells, counted in the static triangles read.
  #[serde(default)]
  pub instances: usize,
  #[serde(default)]
  pub instance_objects: usize,
  #[serde(default)]
  pub instance_triangles: usize,
  /// Impostors standing for trees afar, which the build makes again from the trees.
  pub impostors: usize,
  /// Tree models given their impostor strips back, cut from the level's atlas; a library tree's named for it.
  #[serde(default)]
  pub tree_impostors: usize,
  /// Static triangles dropped as copies of wall marks ([`LevelDecalIndex`](crate::parts::LevelDecalIndex)), counted
  /// in the static triangles read.
  pub decal_triangles: usize,
  /// Static and collision-only triangles dropped as needles, which xrLC stops on; the static ones are counted in the
  /// static triangles read.
  #[serde(default)]
  pub needle_triangles: usize,
  /// Sectors drawing no static geometry, such as one of trees alone, which no shell stands for.
  pub empty_sectors: usize,
  /// Faces of the collision form, where the level's directory held one.
  pub collision_faces: usize,
  /// Collision faces only a tree's triangles lie on: trunks, which come back with their trees.
  pub tree_collision_faces: usize,
  /// Collision faces only a placed library object's collision-only triangles lie on, which come back with it.
  #[serde(default)]
  pub instance_collision_faces: usize,
  /// Collision faces no render triangle lies on, which the collision object holds.
  pub collision_only_faces: usize,
  /// Polygons of `level.hom` and `level.som`, which `<level>_hom` and `<level>_som` hold.
  pub occluder_polygons: usize,
  pub sound_occluder_polygons: usize,
  /// Sound occlusion factors no game material of the library has, whose polygons take `default`.
  pub unmatched_sound_occlusion_factors: usize,
  /// Ladders folded back into objects, and those of them whose packet kept no game material.
  #[serde(default)]
  pub ladders: usize,
  #[serde(default)]
  pub ladders_of_default_material: usize,
  /// Breakables folded back into objects, and those whose part the level's directory does not hold, which stay
  /// entities.
  #[serde(default)]
  pub breakables: usize,
  #[serde(default)]
  pub breakables_without_part: usize,
}
