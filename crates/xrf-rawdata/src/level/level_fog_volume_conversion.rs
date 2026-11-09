use std::collections::BTreeMap;

use xrf_editable_level::{
  EditableCustomObject, EditableFogVolumeKind, EditableFogVolumeObject, EditableObjectPart, EditableObjectTransform,
  EditableShape,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelFogVolFile, LevelFogVolume};
use xrf_math::Matrix4x4;

use crate::level::level_decompile_context::LevelDecompileContext;

/// A level's volumetric fog, between the scene's `fog_volume.part` and the compiled `level.fog_vol`, which the build
/// writes for every level (`BuilderGame.cpp:90`), with no group for a scene of none.
pub struct LevelFogVolumeConversion;

impl LevelFogVolumeConversion {
  /// The editor's prefix for a new volume's name, the tool's class name.
  pub const NAME_PREFIX: &'static str = "fog_volume";

  /// What ends a profile: `IWriter::w_string` writes a line.
  pub const PROFILE_TERMINATOR: &'static str = "\r\n";

  /// The fog volume part a `level.fog_vol` was exported from: per body a group numbered from 1, as the editor numbers
  /// them, holding its emitter and an occlusion volume per obstacle, each named as the editor names a new volume and
  /// standing at the saved transform composing into its matrix.
  ///
  /// # Errors
  ///
  /// Returns an error naming a body whose profile ends otherwise than the export ends one, or a matrix no object
  /// stands at.
  pub fn decompile(
    file: &LevelFogVolFile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<EditableObjectPart<EditableFogVolumeObject>> {
    let mut objects: Vec<EditableFogVolumeObject> = Vec::new();

    for (index, (group, body)) in (1..).zip(&file.volumes).enumerate() {
      if body.terminator != Self::PROFILE_TERMINATOR {
        return Err(XrfError::new_invalid_error(format!(
          "Fog body {index} '{}' ends its profile with {:?}, not as the export writes a line",
          body.profile, body.terminator
        )));
      }

      let volume = |kind: EditableFogVolumeKind, matrix: &Matrix4x4, context: &mut LevelDecompileContext| {
        let transform: EditableObjectTransform =
          EditableObjectTransform::from_matrix_as_saved(matrix).map_err(|error| {
            XrfError::new_invalid_error(format!(
              "Fog body {index} '{}' holds a volume no object stands at: {error}",
              body.profile
            ))
          })?;

        Ok::<_, XrfError>(EditableFogVolumeObject {
          object: EditableCustomObject::new(context.names.get_next(Self::NAME_PREFIX), transform),
          shape: EditableShape::new_unit_box(),
          kind,
          group,
        })
      };

      objects.push(volume(
        EditableFogVolumeKind::Emitter {
          profile: body.profile.clone(),
        },
        &body.transform,
        context,
      )?);

      for obstacle in &body.obstacles {
        objects.push(volume(EditableFogVolumeKind::Occlusion, obstacle, context)?);
      }
    }

    Ok(EditableObjectPart::new(context.guid, objects))
  }

  /// The `level.fog_vol` the build writes from the part: version 3, then per group in ascending number its emitter's
  /// profile and transform and its occlusion volumes' transforms. Within a group the editor sorts volumes by type with
  /// `std::sort`, which keeps no order among the occlusions; this keeps the part's.
  ///
  /// # Errors
  ///
  /// Returns an error naming a group that is not one emitter and its occlusion volumes, for which the build fails.
  pub fn export(part: &EditableObjectPart<EditableFogVolumeObject>) -> XrfResult<LevelFogVolFile> {
    let mut groups: BTreeMap<u32, Vec<&EditableFogVolumeObject>> = BTreeMap::new();

    for volume in &part.objects {
      groups.entry(volume.group).or_default().push(volume);
    }

    let volumes: Vec<LevelFogVolume> = groups
      .into_iter()
      .map(|(group, mut volumes)| {
        let grouping_error = || {
          XrfError::new_invalid_error(format!(
            "Fog volume group {group} is not one emitter and its occlusion volumes"
          ))
        };

        // Emitters first, as `sort_fog_vol` orders `fvEmitter` (0) before `fvOcclusion` (1).
        volumes.sort_by_key(|volume| volume.get_profile().is_none());

        let (emitter, obstacles) = volumes.split_first().ok_or_else(grouping_error)?;
        let profile: &str = emitter
          .get_profile()
          .filter(|_| obstacles.iter().all(|volume| volume.get_profile().is_none()))
          .ok_or_else(grouping_error)?;

        Ok(LevelFogVolume {
          profile: profile.to_owned(),
          terminator: Self::PROFILE_TERMINATOR.to_owned(),
          transform: emitter.object.transform.to_matrix(),
          obstacles: obstacles
            .iter()
            .map(|volume| volume.object.transform.to_matrix())
            .collect(),
        })
      })
      .collect::<XrfResult<Vec<LevelFogVolume>>>()?;

    Ok(LevelFogVolFile {
      version: LevelFogVolFile::CURRENT_VERSION,
      volumes,
    })
  }
}
