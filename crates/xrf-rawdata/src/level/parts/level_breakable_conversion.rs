use xrf_editable_level::EditableObjectTransform;
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_ogf::OgfFile;
use xrf_spawn::{AlifeObject, AlifeObjectInherited};

use crate::level::geometry::level_decompiled_object::LevelDecompiledObject;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A level's breakable parts folded back into the geometry they were built from.
///
/// The Build exports each breakable part (`ESceneObjectTool::ExportBreakableObjects`) as a rigid skeleton
/// `meshes\brkbl#<N>.ogf` in its box's frame, placed by a `breakable_object` at the box. The part comes back as a
/// static object of its faces ([`ObjectDecompiler::decompile_breakable`]) placed as the packet placed it, which the
/// Build cuts out again.
pub struct LevelBreakableConversion;

impl LevelBreakableConversion {
  /// The level's directory the parts are written to, and how their names begin and end.
  pub const DIRECTORY: &'static str = "meshes";
  const PREFIX: &'static str = "brkbl#";
  const EXTENSION: &'static str = ".ogf";

  /// How every part's path in the level's directory begins: the Build owns these files, since a build of fewer parts
  /// would leave the rest for the game to load.
  pub(crate) const FAMILY: &'static str = "meshes/brkbl#";

  /// Whether a file name in the parts' directory is a part the Build exported.
  pub fn is_part_file(name: &str) -> bool {
    name.starts_with(Self::PREFIX) && name.ends_with(Self::EXTENSION)
  }

  /// The Build's name for part `index`, `meshes\brkbl#<index>`, which its entity and visual take, and its path in the
  /// level's directory, `meshes/brkbl#<index>.ogf`.
  pub(crate) fn get_part_names(index: usize) -> (String, String) {
    (
      format!("{}\\{}{index}", Self::DIRECTORY, Self::PREFIX),
      format!("{}/{}{index}{}", Self::DIRECTORY, Self::PREFIX, Self::EXTENSION),
    )
  }

  /// The path of the part a breakable's packet names, `meshes/brkbl#<N>.ogf`; `None` for an entity that is no
  /// breakable or names no part. The packet keeps the name lowered and without its extension
  /// (`CSE_Visual::set_visual`).
  pub fn get_part_path(entity: &AlifeObject) -> Option<String> {
    let AlifeObjectInherited::CseAlifeObjectBreakable(breakable) = &entity.inherited else {
      return None;
    };
    let visual: &str = &breakable.base.visual_name;
    let visual: &str = visual.strip_suffix(Self::EXTENSION).unwrap_or(visual);
    let (directory, name) = visual.split_once('\\')?;

    (directory == Self::DIRECTORY && name.starts_with(Self::PREFIX))
      .then(|| format!("{directory}/{name}{}", Self::EXTENSION))
  }

  /// The breakable `entity`'s part, read from `path`, as `levels\<level>\<level>_breakable_<index>` placed where its
  /// packet placed it.
  ///
  /// # Errors
  ///
  /// Returns an error when the part is no rigid skeleton or its geometry does not hold together.
  pub fn decompile(
    name: &str,
    index: usize,
    entity: &AlifeObject,
    path: &str,
    part: &OgfFile,
  ) -> XrfResult<LevelDecompiledObject> {
    Ok(LevelDecompiledObject {
      reference: format!("levels\\{name}\\{name}_breakable_{index}"),
      transform: EditableObjectTransform {
        position: entity.header.position,
        // `setXYZ` of the packet's angles, as the engine places the part and a scene object's rotation composes.
        rotation: entity.header.direction,
        scale: Vector3d::ONE,
      },
      source: path.to_owned(),
      decompile: ObjectDecompiler::decompile_breakable(part)?,
    })
  }
}
