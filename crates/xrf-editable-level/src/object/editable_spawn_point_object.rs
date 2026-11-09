use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_shape_object::EditableShapeObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

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
  /// into `level.spawn`, or `level_rs.spawn` when [`EditableSpawnPointObject::RESPAWN_ENTITY`] is set.
  Entity {
    /// The section the entity is created from, `name`.
    section: String,
    /// `fl`: whether the entity respawns in multiplayer.
    flags: u8,
    /// The packet `Spawn_Write` puts through `SIniFileStream`, a numbered field per value, kept as stored.
    data: IniSection,
  },
}

/// A spawn point, `CSpawnPoint`: an object of `spawn.part`, with a shape attached where its entity needs a volume.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableSpawnPointObject {
  pub object: EditableCustomObject,
  pub kind: EditableSpawnPointKind,
  /// The one object `SaveObjectsLTX` writes under `attached`, a shape for every spawn point the editor attaches to.
  pub attached: Option<EditableShapeObject>,
}

impl EditableSpawnPointObject {
  /// `SPAWNPOINT_VERSION`, which the editor writes.
  pub const VERSION: u32 = 0x17;

  /// The oldest version `LoadLTX` reads.
  pub const OLDEST_VERSION: u32 = 0x14;

  /// The version from which a respawn point keeps its profile.
  pub const PROFILE_VERSION: u32 = 0x17;

  /// The version from which an environment modifier keeps its flags; an older one mixes in every value.
  pub const FLAGS_VERSION: u32 = 0x16;

  /// `ptRPoint`, `ptEnvMod` and `ptSpawnPoint`, the `type` field.
  pub const RESPAWN_POINT: u32 = 0;
  pub const ENVIRONMENT_MODIFIER: u32 = 1;
  pub const ENTITY: u32 = 2;

  /// The values of a new environment modifier (`CSpawnPoint::Construct`), its flags all set.
  pub const DEFAULT_ENVIRONMENT_FLAGS: u16 = u16::MAX;

  /// `eSDTypeRespawn`, the entity flag (the editor's "MP respawn") that exports it into `level_rs.spawn`.
  pub const RESPAWN_ENTITY: u8 = 1;

  /// The keys `SSpawnData::SaveLTX` writes beside the packet's numbered fields: the entity's section and its flags.
  const ENTITY_SECTION: &'static str = "name";
  const ENTITY_FLAGS: &'static str = "fl";

  /// What the attached objects' section names and count take, `SaveObjectsLTX(..., "attached", ...)`.
  const ATTACHED: &'static str = "attached";

  /// The `type` the kind is saved as.
  pub const fn get_type(&self) -> u32 {
    match self.kind {
      EditableSpawnPointKind::RespawnPoint { .. } => Self::RESPAWN_POINT,
      EditableSpawnPointKind::EnvironmentModifier { .. } => Self::ENVIRONMENT_MODIFIER,
      EditableSpawnPointKind::Entity { .. } => Self::ENTITY,
    }
  }

  fn read_kind(object: &EditablePartObject<'_>, version: u32) -> XrfResult<EditableSpawnPointKind> {
    let section: &IniSection = object.section;
    let kind: u32 = EditableLevelValue::read_u32(section, "type")?;

    match kind {
      Self::RESPAWN_POINT => {
        if version == Self::OLDEST_VERSION {
          return Err(XrfError::new_not_implemented_error(format!(
            "Respawn point [{}] keeps its game type in version {version}'s single mode, which is not read",
            section.name
          )));
        }

        Ok(EditableSpawnPointKind::RespawnPoint {
          team: section.get_parsed::<u8>("team_id")?,
          kind: section.get_parsed::<u8>("rp_type")?,
          game_types: section.get_parsed::<u16>("game_type")?,
          profile: if version >= Self::PROFILE_VERSION {
            section.get_required("rp_profile")?.to_owned()
          } else {
            String::new()
          },
        })
      }
      Self::ENVIRONMENT_MODIFIER => Ok(EditableSpawnPointKind::EnvironmentModifier {
        radius: section.get_parsed::<f32>("em_radius")?,
        power: section.get_parsed::<f32>("em_power")?,
        view_distance: section.get_parsed::<f32>("view_dist")?,
        fog_color: EditableLevelValue::read_u32(section, "fog_color")?,
        fog_density: section.get_parsed::<f32>("fog_density")?,
        ambient_color: EditableLevelValue::read_u32(section, "ambient_color")?,
        sky_color: EditableLevelValue::read_u32(section, "sky_color")?,
        hemi_color: EditableLevelValue::read_u32(section, "hemi_color")?,
        flags: if version >= Self::FLAGS_VERSION {
          section.get_parsed::<u16>("em_flags")?
        } else {
          Self::DEFAULT_ENVIRONMENT_FLAGS
        },
      }),
      Self::ENTITY => {
        let mut data: IniSection = object
          .get_child("spawndata")
          .cloned()
          .ok_or_else(|| XrfError::new_parsing_error(format!("Spawn point [{}] has no spawn data", section.name)))?;
        let entity: String = data.get_required(Self::ENTITY_SECTION)?.to_owned();
        // `LoadLTX` keeps a new entity's flags where an older scene saved none.
        let flags: u8 = if data.get(Self::ENTITY_FLAGS).is_some() {
          data.get_parsed::<u8>(Self::ENTITY_FLAGS)?
        } else {
          0
        };

        data.remove(Self::ENTITY_SECTION);
        data.remove(Self::ENTITY_FLAGS);

        Ok(EditableSpawnPointKind::Entity {
          section: entity,
          flags,
          data,
        })
      }
      kind => Err(XrfError::new_parsing_error(format!(
        "Spawn point [{}] is of type {kind}, which the editor does not know",
        section.name
      ))),
    }
  }

  fn read_attached(object: &EditablePartObject<'_>) -> XrfResult<Option<EditableShapeObject>> {
    let section: &IniSection = object.section;
    let key: String = format!("{}_count", Self::ATTACHED);

    if section.get(&key).is_none() {
      return Ok(None);
    }

    match EditableLevelValue::read_u32(section, &key)? {
      0 => Ok(None),
      1 => {
        let attached: &IniSection = object.get_child(&format!("{}_0", Self::ATTACHED)).ok_or_else(|| {
          XrfError::new_parsing_error(format!(
            "Spawn point [{}] counts an attachment it does not hold",
            section.name
          ))
        })?;
        let class_id: i32 = attached.get_parsed::<i32>("clsid")?;

        if class_id != EditableShapeObject::CLASS_ID {
          return Err(XrfError::new_parsing_error(format!(
            "Spawn point [{}] holds an attachment of class {class_id}, not a shape",
            section.name
          )));
        }

        Ok(Some(EditableShapeObject::read_from_section(attached)?))
      }
      count => Err(XrfError::new_parsing_error(format!(
        "Spawn point [{}] counts {count} attachments, where the editor keeps one",
        section.name
      ))),
    }
  }
}

impl EditableToolObject for EditableSpawnPointObject {
  /// `OBJCLASS_SPAWNPOINT`.
  const CLASS_ID: i32 = 6;
  const TOOL_NAME: &'static str = "spawn";

  /// The spawn tool keeps flags (`ESceneSpawnToolsIO.cpp:28`).
  const KEEPS_FLAGS: bool = true;

  /// As `CSpawnPoint::LoadLTX` reads it, from version 0x15; version 0x14's single-mode game type is refused.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = EditableLevelValue::read_u32(section, "version")?;

    if version < Self::OLDEST_VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Spawn point [{}] is of version {version}, older than the editor reads",
        section.name
      )));
    }

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      kind: Self::read_kind(object, version)?,
      attached: Self::read_attached(object)?,
    })
  }

  /// As `CSpawnPoint::SaveLTX` writes it, at [`Self::VERSION`]: its attachment under `attached`, then its kind.
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    if let Some(attached) = &self.attached {
      let name: String = format!("{section}_{}_0", Self::ATTACHED);
      let attached_section: &mut IniSection = ini.section_mut(&name);

      attached_section.set("clsid", EditableShapeObject::CLASS_ID.to_string());
      attached.write_to_section(attached_section);
    }

    if let EditableSpawnPointKind::Entity {
      section: entity,
      flags,
      data,
    } = &self.kind
    {
      let spawn_data: &mut IniSection = ini.section_mut(&format!("{section}_spawndata"));

      for field in &data.fields {
        spawn_data.set(field.key.clone(), field.value.clone());
      }

      spawn_data.set(Self::ENTITY_FLAGS, flags.to_string());
      spawn_data.set(Self::ENTITY_SECTION, entity.clone());
    }

    let object: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(object);
    object.set("version", Self::VERSION.to_string());

    if self.attached.is_some() {
      object.set(format!("{}_count", Self::ATTACHED), "1");
    }

    object.set("type", self.get_type().to_string());

    match &self.kind {
      EditableSpawnPointKind::RespawnPoint {
        team,
        kind,
        game_types,
        profile,
      } => {
        object.set("team_id", team.to_string());
        object.set("rp_profile", profile.clone());
        object.set("rp_type", kind.to_string());
        object.set("game_type", game_types.to_string());
      }
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
      } => {
        object.set("em_radius", EditableLevelValue::format_f32(*radius));
        object.set("em_power", EditableLevelValue::format_f32(*power));
        object.set("view_dist", EditableLevelValue::format_f32(*view_distance));
        object.set("fog_color", EditableLevelValue::format_u32(*fog_color));
        object.set("fog_density", EditableLevelValue::format_f32(*fog_density));
        object.set("ambient_color", EditableLevelValue::format_u32(*ambient_color));
        object.set("sky_color", EditableLevelValue::format_u32(*sky_color));
        object.set("hemi_color", EditableLevelValue::format_u32(*hemi_color));
        object.set("em_flags", flags.to_string());
      }
      EditableSpawnPointKind::Entity { .. } => {}
    }
  }
}
