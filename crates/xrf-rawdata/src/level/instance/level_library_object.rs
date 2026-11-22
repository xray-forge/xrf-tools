use std::path::PathBuf;

use xrf_math::glam::DVec3;

use crate::level::instance::level_library_triangle::LevelLibraryTriangle;

/// One object of an object library as instance recovery looks for it in a compiled level: the triangles of its
/// surfaces that render, in its own space, which the Build moves into the level by the scene object's transform
/// (`SceneBuilder::BuildMesh`) and xrLC keeps as drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelLibraryObject {
  /// Its path under the library's root without the extension, folders joined by backslashes: what a scene object's
  /// `reference_name` names.
  pub reference: String,
  pub path: PathBuf,
  /// Its meshes' names, each of which the sector holding that mesh's triangles of a placement names.
  pub meshes: Vec<String>,
  /// Its rendered triangles, the largest first.
  pub triangles: Vec<LevelLibraryTriangle>,
  /// Its triangles that collide without rendering, which xrLC writes to the collision form alone.
  pub collision: Vec<[DVec3; 3]>,
  /// The rendered triangles' area, square metres.
  pub area: f64,
}
