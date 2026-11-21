use serde::Serialize;

/// What a build took from the scene and left out.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildCensus {
  /// Static objects whose meshes went into the geometry, multi-use objects aside: their placements are
  /// [`Self::multi_use_references`].
  pub static_objects: usize,
  /// Objects of no static geometry: dynamic ones, and occluders, which `level.hom` and `level.som` take. Multi-use
  /// objects are counted in [`Self::multi_use_references`], not here.
  pub other_objects: usize,
  pub faces: usize,
  /// Faces of occluder, breakable and climbable surfaces, which other exports take.
  pub skipped_faces: usize,
  /// Faces of no area left out: the objects' faces, which the Build drops, and the wall marks' triangles, which xrLC
  /// would stop on.
  pub invalid_faces: usize,
  pub static_lights: usize,
  pub dynamic_lights: usize,
  pub glows: usize,
  pub portals: usize,
  pub occluder_polygons: usize,
  pub sound_occluder_polygons: usize,
  /// Multi-use models and the placements referencing them; with the static and other objects, every object placed.
  pub multi_use_models: usize,
  pub multi_use_references: usize,
  /// Multi-use models drawn from afar as impostors.
  pub impostors: usize,
  pub ai_map_nodes: usize,
  /// Detail objects the slots plant, and the textures their atlas merges.
  pub detail_objects: usize,
  pub detail_textures: usize,
  /// Breakables and ladders made of breakable and climbable faces, and parts of either refused.
  pub breakable_objects: usize,
  pub climbable_objects: usize,
  pub refused_parts: usize,
}
