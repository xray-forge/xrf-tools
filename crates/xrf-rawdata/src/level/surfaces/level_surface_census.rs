use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::level::surfaces::level_compiler_shader_choice::LevelCompilerShaderChoice;

/// What the surfaces of a level's decompile were dressed from, counted.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSurfaceCensus {
  /// Triangles lying on a collision face of the very same three points.
  pub exact_triangles: usize,
  /// Triangles a ray from just above found a collision face beneath, as the level compiler splits render faces it
  /// does not split in the collision form.
  pub traced_triangles: usize,
  /// Triangles on no collision face, which the level compiler built from a compiler shader of no collision.
  pub uncollided_triangles: usize,
  /// Collision materials the game material library does not define, by number; their faces take `default`.
  pub unknown_game_materials: BTreeSet<u16>,
  pub compiler_shaders: Vec<LevelCompilerShaderChoice>,
}
