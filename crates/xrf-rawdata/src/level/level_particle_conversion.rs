use xrf_editable_level::{
  EditableCustomObject, EditableLevelGuid, EditableObjectPart, EditableObjectTransform, EditableParticleObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelPsStaticFile, LevelPsStaticPlacement};

use crate::level::level_object_names::LevelObjectNames;

/// A level's planted particle effects, between the scene's `ps.part` and the compiled `level.ps_static`, which the
/// build passes through with no compiler (`BuilderGame.cpp:81`).
pub struct LevelParticleConversion;

impl LevelParticleConversion {
  /// `PS_TOOLS_VERSION`, the particle tool's `[main]` version.
  pub const TOOL_VERSION: u32 = 0;

  /// The particle part a `level.ps_static` was exported from, each placement an object named after its effect, its
  /// transform rounded as the part's text holds it. A file of version 0 keeps no game modes, so its effects play in all
  /// of them.
  ///
  /// # Errors
  ///
  /// Returns an error naming the placement whose transform is not one an object can stand at.
  pub fn decompile(
    file: &LevelPsStaticFile,
    guid: EditableLevelGuid,
    names: &mut LevelObjectNames,
  ) -> XrfResult<EditableObjectPart<EditableParticleObject>> {
    let objects: Vec<EditableParticleObject> = file
      .placements
      .iter()
      .enumerate()
      .map(|(index, placement)| {
        let transform: EditableObjectTransform =
          EditableObjectTransform::from_matrix(&placement.transform).map_err(|error| {
            XrfError::new_invalid_error(format!(
              "Static particle {index} '{}' does not stand as an object: {error}",
              placement.effect
            ))
          })?;
        let stem: &str = placement.effect.rsplit(['\\', '/']).next().unwrap_or_default();

        Ok(EditableParticleObject {
          object: EditableCustomObject::new(names.get_next(stem), transform.round_as_saved()),
          reference: placement.effect.clone(),
          game_types: placement.game_types.unwrap_or(EditableParticleObject::ALL_GAME_TYPES),
        })
      })
      .collect::<XrfResult<Vec<EditableParticleObject>>>()?;

    Ok(EditableObjectPart::new(guid, Self::TOOL_VERSION, None, objects))
  }

  /// The `level.ps_static` the build writes from the part, `EParticlesObject::ExportGame`: version 1, then each
  /// object's game modes, effect and transform in the part's order. None for a part of no objects, for which the build
  /// writes no file.
  pub fn export(part: &EditableObjectPart<EditableParticleObject>) -> Option<LevelPsStaticFile> {
    (!part.objects.is_empty()).then(|| LevelPsStaticFile {
      version: LevelPsStaticFile::CURRENT_VERSION,
      placements: part
        .objects
        .iter()
        .map(|object| LevelPsStaticPlacement {
          game_types: Some(object.game_types),
          effect: object.reference.clone(),
          transform: object.object.transform.to_matrix(),
        })
        .collect(),
    })
  }
}
