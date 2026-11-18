use xrf_editable_level::EditableObjectTransform;

use crate::object::decompile::object_decompile::ObjectDecompile;

/// One object the decompile writes, by the path the scene references it at, and where its scene object stands.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelDecompiledObject {
  /// Its path under `$objects$` without the extension, such as `levels\<level>\<level>_sector_<N>`: what the scene
  /// object's `reference_name` names.
  pub reference: String,
  /// The origin for a shell, the collision or an occluder; where its packet placed it for a ladder or breakable part.
  pub transform: EditableObjectTransform,
  /// The compiled file it came from, by its path in the level's directory, as its sidecar records, such as
  /// `level.geom` or `meshes/brkbl#<N>.ogf`.
  pub source: String,
  pub decompile: ObjectDecompile,
}
