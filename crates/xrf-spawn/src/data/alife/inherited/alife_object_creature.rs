use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;
use xrf_utils::{vector_from_string, vector_to_string};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;

/// A creature, `CSE_ALifeCreatureAbstract`. Its update packet repeats its health, team, squad and group, and carries
/// where it stands and which way its torso faces; the engine reads them over the spawn packet's, so a creature starts
/// where its update packet says.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectCreature {
  pub base: AlifeObjectDynamicVisual,
  pub team: u8,
  pub squad: u8,
  pub group: u8,
  pub health: f32,
  pub dynamic_out_restrictions: Vec<u16>,
  pub dynamic_in_restrictions: Vec<u16>,
  pub killer_id: u16,
  pub game_death_time: u64,
  pub update_health: f32,
  pub update_timestamp: u32,
  pub update_flags: u8,
  /// Where the creature stands, `o_Position`.
  pub update_position: Vector3d,
  /// The model's heading, `o_model`.
  pub update_model_yaw: f32,
  /// The torso's heading, `o_torso.yaw`, which the spawn packet's direction gives as its y.
  pub update_torso_yaw: f32,
  /// The torso's pitch, `o_torso.pitch`, which the spawn packet's direction gives as its x.
  pub update_torso_pitch: f32,
  pub update_torso_roll: f32,
  pub update_team: u8,
  pub update_squad: u8,
  pub update_group: u8,
}

impl AlifeObjectCreature {
  /// Place the creature at `position`, its torso facing `direction` as the spawn packet's angles give it: pitch about
  /// x and heading about y, as `CSE_ALifeCreatureAbstract::STATE_Read` derives the torso from `o_Angle`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.update_position = position.clone();
    self.update_torso_pitch = direction.x;
    self.update_torso_yaw = direction.y;
  }
}

impl AlifePacketsReadWrite for AlifeObjectCreature {
  /// Read a creature from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: spawn.read_xr::<T, _>()?,
      team: spawn.read_u8()?,
      squad: spawn.read_u8()?,
      group: spawn.read_u8()?,
      health: spawn.read_f32::<T>()?,
      dynamic_out_restrictions: spawn.read_u16_vector::<T>()?,
      dynamic_in_restrictions: spawn.read_u16_vector::<T>()?,
      killer_id: spawn.read_u16::<T>()?,
      game_death_time: spawn.read_u64::<T>()?,
      update_health: update.read_f32::<T>()?,
      update_timestamp: update.read_u32::<T>()?,
      update_flags: update.read_u8()?,
      update_position: update.read_xr::<T, _>()?,
      update_model_yaw: update.read_f32::<T>()?,
      update_torso_yaw: update.read_f32::<T>()?,
      update_torso_pitch: update.read_f32::<T>()?,
      update_torso_roll: update.read_f32::<T>()?,
      update_team: update.read_u8()?,
      update_squad: update.read_u8()?,
      update_group: update.read_u8()?,
    })
  }

  /// Write a creature into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    spawn.write_xr::<T, _>(&self.base)?;

    spawn.write_u8(self.team)?;
    spawn.write_u8(self.squad)?;
    spawn.write_u8(self.group)?;
    spawn.write_f32::<T>(self.health)?;

    spawn.write_u16_vector::<T>(&self.dynamic_out_restrictions)?;
    spawn.write_u16_vector::<T>(&self.dynamic_in_restrictions)?;

    spawn.write_u16::<T>(self.killer_id)?;
    spawn.write_u64::<T>(self.game_death_time)?;

    update.write_f32::<T>(self.update_health)?;
    update.write_u32::<T>(self.update_timestamp)?;
    update.write_u8(self.update_flags)?;
    update.write_xr::<T, _>(&self.update_position)?;
    update.write_f32::<T>(self.update_model_yaw)?;
    update.write_f32::<T>(self.update_torso_yaw)?;
    update.write_f32::<T>(self.update_torso_pitch)?;
    update.write_f32::<T>(self.update_torso_roll)?;
    update.write_u8(self.update_team)?;
    update.write_u8(self.update_squad)?;
    update.write_u8(self.update_group)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectCreature {
  /// Import ALife creature object from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      team: section.read_field("creature.team")?,
      squad: section.read_field("creature.squad")?,
      group: section.read_field("creature.group")?,
      health: section.read_field("creature.health")?,
      dynamic_out_restrictions: vector_from_string(
        &section.read_field::<String>("creature.dynamic_out_restrictions")?,
      )?,
      dynamic_in_restrictions: vector_from_string(&section.read_field::<String>("creature.dynamic_in_restrictions")?)?,
      killer_id: section.read_field("creature.killer_id")?,
      game_death_time: section.read_field("creature.game_death_time")?,
      update_health: section.read_field("creature.update_health")?,
      update_timestamp: section.read_field("creature.update_timestamp")?,
      update_flags: section.read_field("creature.update_flags")?,
      update_position: section.read_field("creature.update_position")?,
      update_model_yaw: section.read_field("creature.update_model_yaw")?,
      update_torso_yaw: section.read_field("creature.update_torso_yaw")?,
      update_torso_pitch: section.read_field("creature.update_torso_pitch")?,
      update_torso_roll: section.read_field("creature.update_torso_roll")?,
      update_team: section.read_field("creature.update_team")?,
      update_squad: section.read_field("creature.update_squad")?,
      update_group: section.read_field("creature.update_group")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("creature.team", self.team.to_string())
      .set("creature.squad", self.squad.to_string())
      .set("creature.group", self.group.to_string())
      .set("creature.health", self.health.to_string())
      .set(
        "creature.dynamic_out_restrictions",
        vector_to_string(&self.dynamic_out_restrictions),
      )
      .set(
        "creature.dynamic_in_restrictions",
        vector_to_string(&self.dynamic_in_restrictions),
      )
      .set("creature.killer_id", self.killer_id.to_string())
      .set("creature.game_death_time", self.game_death_time.to_string())
      .set("creature.update_health", self.update_health.to_string())
      .set("creature.update_timestamp", self.update_timestamp.to_string())
      .set("creature.update_flags", self.update_flags.to_string())
      .set("creature.update_position", self.update_position.to_string())
      .set("creature.update_model_yaw", self.update_model_yaw.to_string())
      .set("creature.update_torso_yaw", self.update_torso_yaw.to_string())
      .set("creature.update_torso_pitch", self.update_torso_pitch.to_string())
      .set("creature.update_torso_roll", self.update_torso_roll.to_string())
      .set("creature.update_team", self.update_team.to_string())
      .set("creature.update_squad", self.update_squad.to_string())
      .set("creature.update_group", self.update_group.to_string());

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectCreature {
  /// A living creature at `position`, facing along its spawn packet's direction as the SDK leaves one.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 621,
          distance: 0.0,
          direct_control: 1,
          level_vertex_id: 52235,
          flags: 0,
          custom_data: String::new(),
          story_id: AlifeObjectAbstract::INVALID_STORY_ID,
          spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
        },
        visual_name: String::from("actors\\stalker_hero\\stalker_hero_1"),
        visual_flags: 0,
      },
      team: 0,
      squad: 0,
      group: 0,
      health: 1.0,
      dynamic_out_restrictions: Vec::new(),
      dynamic_in_restrictions: Vec::new(),
      killer_id: u16::MAX,
      game_death_time: 0,
      update_health: 1.0,
      update_timestamp: 0,
      update_flags: 0,
      update_position: position,
      update_model_yaw: 0.0,
      update_torso_yaw: 2.81,
      update_torso_pitch: -0.012,
      update_torso_roll: 0.0,
      update_team: 0,
      update_squad: 0,
      update_group: 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;

  fn new_object() -> AlifeObjectCreature {
    let mut object: AlifeObjectCreature = AlifeObjectCreature::new_mock(Vector3d::new(256.24, 19.81, 550.82));

    object.dynamic_out_restrictions = vec![1, 2, 3, 4];
    object.dynamic_in_restrictions = vec![5, 6, 7, 8];
    object.update_health = 0.5;
    object.update_timestamp = 0xc2ba_3333;
    object.update_flags = 0x99;

    object
  }

  /// Health, timestamp, flags, position, four angles and three bytes: `CSE_ALifeCreatureAbstract::UPDATE_Write`.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectCreature = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (104, 40, original));

    Ok(())
  }

  #[test]
  fn moves_where_it_stands_and_turns_its_torso() {
    let mut creature: AlifeObjectCreature = new_object();

    creature.move_to(&Vector3d::new(1.0, 2.0, 3.0), &Vector3d::new(-0.25, 1.5, 0.75));

    assert_eq!(creature.update_position, Vector3d::new(1.0, 2.0, 3.0));
    assert_eq!(creature.update_torso_pitch, -0.25);
    assert_eq!(creature.update_torso_yaw, 1.5);
    // The model's heading and the torso's roll are not what the spawn packet's direction gives.
    assert_eq!(creature.update_model_yaw, 0.0);
    assert_eq!(creature.update_torso_roll, 0.0);
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectCreature = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectCreature::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectCreature = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectCreature>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
