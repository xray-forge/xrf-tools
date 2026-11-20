use std::path::PathBuf;

use xrf_editable_level::EditableObjectTransform;

use crate::level::instance::level_instance_mesh::LevelInstanceMesh;

/// A library object a decompile found placed in the level's static geometry, which the scene places by reference.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelInstancePlacement {
  /// The library object, by the reference its scene object names, and the file it was read from.
  pub reference: String,
  pub path: PathBuf,
  /// The scene object placing it, named once the scene names its objects.
  pub object: String,
  /// Its meshes, each with the compiled sector that names it.
  pub meshes: Vec<LevelInstanceMesh>,
  pub transform: EditableObjectTransform,
  /// How much of its rendered area the level drew as its own triangles, and how far its placed corners stood from the
  /// level's, metres.
  pub coverage: f64,
  pub error: f64,
  /// Level triangles its placement took out of the shells.
  pub triangles: usize,
}
