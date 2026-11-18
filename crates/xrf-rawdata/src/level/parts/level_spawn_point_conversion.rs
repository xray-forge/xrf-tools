use xrf_editable_level::{
  EditableCustomObject, EditableObjectPart, EditableObjectTransform, EditableSpawnPointKind, EditableSpawnPointObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelEnvModFile, LevelEnvModifier, LevelGameRPoint};
use xrf_math::Vector3d;

use crate::level::level_decompile_context::LevelDecompileContext;

/// A level's respawn points and environment modifiers, between the scene's `spawn.part` and the respawn point table of
/// `level.game` and `level.env_mod`, which `CSpawnPoint::ExportGame` (`SpawnPoint.cpp:1328`) fills. The part's entity
/// spawns, which the build exports into `level.spawn`, are converted on their own.
pub struct LevelSpawnPointConversion;

impl LevelSpawnPointConversion {
  /// The prefixes the decompile names each kind after.
  pub const RESPAWN_POINT_PREFIX: &'static str = "rpoint";
  pub const ENVIRONMENT_MODIFIER_PREFIX: &'static str = "env_mod";

  /// The spawn part the respawn points and environment modifiers were exported from, the respawn points first, each
  /// at the file's position and turn as a new object stands, named after its kind. A colour comes back as the packed
  /// word whose channels `u32_3f` gives the file's floats from, with no alpha.
  ///
  /// # Errors
  ///
  /// Returns an error naming a modifier whose colour no packed word exports to.
  pub fn decompile(
    respawn_points: &[LevelGameRPoint],
    environment: &LevelEnvModFile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<EditableObjectPart<EditableSpawnPointObject>> {
    let mut objects: Vec<EditableSpawnPointObject> = respawn_points
      .iter()
      .map(|point| EditableSpawnPointObject {
        object: EditableCustomObject::new(
          context.names.get_next(Self::RESPAWN_POINT_PREFIX),
          EditableObjectTransform {
            position: point.position,
            rotation: point.rotation,
            scale: Vector3d::ONE,
          },
        ),
        kind: EditableSpawnPointKind::RespawnPoint {
          team: point.team,
          kind: point.kind,
          game_types: point.game_types,
          profile: point.profile.clone(),
        },
        attached: None,
      })
      .collect();

    for (index, modifier) in environment.modifiers.iter().enumerate() {
      let color = |color: &Vector3d, what: &str| {
        Self::get_packed_color(color).ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "Environment modifier {index}'s {what} {color:?} is not one a packed colour exports to"
          ))
        })
      };

      objects.push(EditableSpawnPointObject {
        object: EditableCustomObject::new(
          context.names.get_next(Self::ENVIRONMENT_MODIFIER_PREFIX),
          EditableObjectTransform {
            position: modifier.position,
            ..EditableObjectTransform::default()
          },
        ),
        kind: EditableSpawnPointKind::EnvironmentModifier {
          radius: modifier.radius,
          power: modifier.power,
          view_distance: modifier.far_plane,
          fog_color: color(&modifier.fog_color, "fog colour")?,
          fog_density: modifier.fog_density,
          ambient_color: color(&modifier.ambient, "ambient colour")?,
          sky_color: color(&modifier.sky_color, "sky colour")?,
          hemi_color: color(&modifier.hemi_color, "hemi colour")?,
          flags: modifier
            .use_flags
            .unwrap_or(EditableSpawnPointObject::DEFAULT_ENVIRONMENT_FLAGS),
        },
        attached: None,
      });
    }

    Ok(EditableObjectPart::new(context.guid, objects))
  }

  /// The respawn point table `ExportGame` fills from the part, a record per respawn point in the part's order: its
  /// position, turn, team, kind, game modes and profile.
  pub fn export_respawn_points(part: &EditableObjectPart<EditableSpawnPointObject>) -> Vec<LevelGameRPoint> {
    part
      .objects
      .iter()
      .filter_map(|point| match &point.kind {
        EditableSpawnPointKind::RespawnPoint {
          team,
          kind,
          game_types,
          profile,
        } => Some(LevelGameRPoint {
          position: point.object.transform.position,
          rotation: point.object.transform.rotation,
          team: *team,
          kind: *kind,
          game_types: *game_types,
          profile: profile.clone(),
        }),
        _ => None,
      })
      .collect()
  }

  /// The `level.env_mod` `BuildGame` writes for every level: `SPAWNPOINT_VERSION` in its first chunk
  /// (`BuilderGame.cpp:21`), then a modifier per environment modifier in the part's order, its colours through
  /// `u32_3f`.
  pub fn export_environment_modifiers(part: &EditableObjectPart<EditableSpawnPointObject>) -> LevelEnvModFile {
    LevelEnvModFile {
      version: EditableSpawnPointObject::VERSION,
      modifiers: part
        .objects
        .iter()
        .filter_map(|point| match &point.kind {
          EditableSpawnPointKind::EnvironmentModifier {
            radius,
            power,
            view_distance,
            fog_color,
            fog_density,
            ambient_color,
            sky_color,
            hemi_color,
            flags,
          } => Some(LevelEnvModifier {
            position: point.object.transform.position,
            radius: *radius,
            power: *power,
            far_plane: *view_distance,
            fog_color: Self::get_color_vector(*fog_color),
            fog_density: *fog_density,
            ambient: Self::get_color_vector(*ambient_color),
            sky_color: Self::get_color_vector(*sky_color),
            hemi_color: Self::get_color_vector(*hemi_color),
            use_flags: Some(*flags),
          }),
          _ => None,
        })
        .collect(),
    }
  }

  /// `u32_3f`: each of red, green and blue times `1 / 255`, in `f32`.
  fn get_color_vector(color: u32) -> Vector3d {
    let [_, red, green, blue] = color.to_be_bytes();

    Vector3d::new(
      Self::get_channel(red),
      Self::get_channel(green),
      Self::get_channel(blue),
    )
  }

  fn get_channel(byte: u8) -> f32 {
    (1.0_f32 / 255.0) * f32::from(byte)
  }

  /// The packed colour of no alpha whose channels [`Self::get_color_vector`] gives `color`'s very bits from.
  fn get_packed_color(color: &Vector3d) -> Option<u32> {
    let byte = |value: f32| (0..=u8::MAX).find(|byte| Self::get_channel(*byte).to_bits() == value.to_bits());

    Some(u32::from_be_bytes([0, byte(color.x)?, byte(color.y)?, byte(color.z)?]))
  }
}
