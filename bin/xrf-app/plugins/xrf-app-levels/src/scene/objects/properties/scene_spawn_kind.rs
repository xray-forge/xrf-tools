use serde::Serialize;
use xrf_editable_level::EditableSpawnPointKind;

use crate::scene::SceneSpawnField;

/// What a spawn point places, with what its kind keeps.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum SceneSpawnKind {
  /// Where a multiplayer team, artefact or item respawns.
  RespawnPoint {
    team: u8,
    /// An actor, artefact or item spawn.
    point_type: u8,
    /// The game modes it answers to, one bit each.
    game_types: u16,
    /// The item preset an item spawn names.
    profile: String,
  },
  /// A local override of the weather.
  EnvironmentModifier {
    radius: f32,
    power: f32,
    view_distance: f32,
    /// Colours as packed `0xAARRGGBB` words.
    fog_color: u32,
    fog_density: f32,
    ambient_color: u32,
    sky_color: u32,
    hemi_color: u32,
    /// Which values the engine mixes in, one bit each.
    flags: u16,
  },
  /// A server entity.
  Entity {
    /// The section the entity is created from.
    section: String,
    /// Whether the entity respawns in multiplayer, `fl`.
    flags: u8,
    /// Its packet's fields in the order the scene saves them.
    data: Vec<SceneSpawnField>,
  },
}

impl From<&EditableSpawnPointKind> for SceneSpawnKind {
  fn from(kind: &EditableSpawnPointKind) -> Self {
    match kind {
      EditableSpawnPointKind::RespawnPoint {
        team,
        kind,
        game_types,
        profile,
      } => Self::RespawnPoint {
        team: *team,
        point_type: *kind,
        game_types: *game_types,
        profile: profile.clone(),
      },
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
      } => Self::EnvironmentModifier {
        radius: *radius,
        power: *power,
        view_distance: *view_distance,
        fog_color: *fog_color,
        fog_density: *fog_density,
        ambient_color: *ambient_color,
        sky_color: *sky_color,
        hemi_color: *hemi_color,
        flags: *flags,
      },
      EditableSpawnPointKind::Entity { section, flags, data } => Self::Entity {
        section: section.clone(),
        flags: *flags,
        data: data
          .fields()
          .iter()
          .map(|field| SceneSpawnField {
            key: field.key.clone(),
            value: field.value.clone(),
          })
          .collect(),
      },
    }
  }
}
