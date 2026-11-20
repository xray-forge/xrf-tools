use xrf_ini::IniSection;

/// What a spawn point places, `CSpawnPoint::EPointType` with its own fields.
#[derive(Clone, Debug, PartialEq)]
pub enum EditableSpawnPointKind {
  /// `ptRPoint`: where a multiplayer team, artefact or item respawns. The build exports it into `level.game`.
  RespawnPoint {
    team: u8,
    /// `ERPpointType`: an actor, artefact or item spawn.
    kind: u8,
    /// The game modes it answers to, one bit each.
    game_types: u16,
    /// The item preset an item spawn names, `rp_profile`.
    profile: String,
  },
  /// `ptEnvMod`: a local override of the weather. The build exports it into `level.env_mod`.
  EnvironmentModifier {
    radius: f32,
    power: f32,
    view_distance: f32,
    /// Colours as packed `0xAARRGGBB` words, which the export turns into three floats each.
    fog_color: u32,
    fog_density: f32,
    ambient_color: u32,
    sky_color: u32,
    hemi_color: u32,
    /// Which values the engine mixes in, one bit each.
    flags: u16,
  },
  /// `ptSpawnPoint`: a server entity, `SSpawnData` as `[object_N_spawndata]` saves it. The build exports its packet
  /// into `level.spawn`, or `level_rs.spawn` when
  /// [`EditableSpawnPointObject::RESPAWN_ENTITY`](crate::EditableSpawnPointObject::RESPAWN_ENTITY) is set.
  Entity {
    /// The section the entity is created from, `name`.
    section: String,
    /// `fl`: whether the entity respawns in multiplayer.
    flags: u8,
    /// The packet `Spawn_Write` puts through `SIniFileStream`, a numbered field per value, kept as stored.
    data: IniSection,
  },
}
