use std::path::PathBuf;

use xrf_editable_level::EditableObjectTransform;

/// A library object a decompile found placed in the level's static geometry, which the scene places by reference.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelInstancePlacement {
  /// The library object, by the reference its scene object names, and the file it was read from.
  pub reference: String,
  pub path: PathBuf,
  /// The scene object placing it, named once the scene names its objects.
  pub object: String,
  /// Its meshes' names, which its sector names.
  pub meshes: Vec<String>,
  pub transform: EditableObjectTransform,
  /// The compiled sector holding most of its triangles, which names its meshes.
  pub sector: u16,
  /// How much of its rendered area the level drew as its own triangles, and how far its placed corners stood from the
  /// level's, metres.
  pub coverage: f64,
  pub error: f64,
  /// Level triangles its placement took out of the shells.
  pub triangles: usize,
}
