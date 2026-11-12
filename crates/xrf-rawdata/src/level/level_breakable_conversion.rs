use xrf_editable_level::EditableObjectTransform;
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_ogf::OgfFile;
use xrf_spawn::{AlifeObject, AlifeObjectInherited};

use crate::level::level_geometry_conversion::LevelDecompiledObject;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A level's breakable parts folded back into the geometry they were built from. The Build leaves the
/// faces of a breakable game material out of the level's geometry and exports each connected part
/// (`ESceneObjectTool::ExportBreakableObjects`) as a rigid skeleton in the level's directory, `meshes\brkbl#<N>.ogf`,
/// its points in the frame of the part's box, and a `breakable_object` naming it, standing at the box with the angles
/// `getXYZ` reads of its axes. The part comes back as a static object of its faces ([`ObjectDecompiler::
/// decompile_breakable`]) that the scene places as the packet placed the part, which the Build cuts out again.
pub struct LevelBreakableConversion;

impl LevelBreakableConversion {
  /// The directory of the level's the parts are written to, and how their names begin and end.
  pub const DIRECTORY: &'static str = "meshes";
  const PREFIX: &'static str = "brkbl#";
  const EXTENSION: &'static str = ".ogf";

  /// Whether a file of the parts' directory, by its name, is a part the Build exported.
  pub fn is_part_file(name: &str) -> bool {
    name.starts_with(Self::PREFIX) && name.ends_with(Self::EXTENSION)
  }

  /// The name the Build gives the part numbered `index`, `meshes\brkbl#<index>`, which its entity and visual take, and
  /// the part's path in the level's directory, `meshes/brkbl#<index>.ogf`.
  pub(crate) fn get_part_names(index: usize) -> (String, String) {
    (
      format!("{}\\{}{index}", Self::DIRECTORY, Self::PREFIX),
      format!("{}/{}{index}{}", Self::DIRECTORY, Self::PREFIX, Self::EXTENSION),
    )
  }

  /// The part a breakable's packet names, by its path in the level's directory, `meshes/brkbl#<N>.ogf`; `None` for an
  /// entity that is no breakable or names a visual of no part. The packet keeps the name the export gave it, lowered
  /// and without its extension (`CSE_Visual::set_visual`).
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

  /// The breakable `entity`, its part read from `path`, as `levels\<level>\<level>_breakable_<index>`, placed where its
  /// packet placed the part.
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
