use serde::Serialize;

/// The surfaces a decompile inferred, counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileSurfaces {
  /// Triangles on a collision face of the same three points.
  pub exact_triangles: usize,
  /// Triangles a short ray found a collision face beneath.
  pub traced_triangles: usize,
  /// Triangles on no collision face.
  pub uncollided_triangles: usize,
  /// Compiler shaders chosen, one per engine shader and kind of evidence.
  pub compiler_shaders: usize,
  /// Collision materials the game material library does not define.
  pub unknown_game_materials: usize,
}
