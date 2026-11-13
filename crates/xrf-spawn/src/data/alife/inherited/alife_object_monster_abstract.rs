use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;

/// A creature the alife simulation moves offline, `CSE_ALifeMonsterAbstract`: every monster and every stalker. Its
/// spawn packet adds the space restrictions it keeps to and the smart terrain it serves, and its update packet the game
/// graph edge it walks, after the creature's fields in each.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectMonsterAbstract {
  pub base: AlifeObjectCreature,
  /// The restrictors it may not leave, `m_out_space_restrictors`, as a comma-separated list of names.
  pub out_space_restrictors: String,
  /// The restrictors it may not enter, `m_in_space_restrictors`.
  pub in_space_restrictors: String,
  /// The smart terrain whose job it holds, `m_smart_terrain_id`; `u16::MAX` for none.
  pub smart_terrain_id: u16,
  /// Whether it reached its smart terrain task, a C++ `bool` stored as one byte.
  pub task_reached: u8,
  /// The game vertex it walks to, `m_tNextGraphID`.
  pub update_next_game_vertex_id: u16,
  /// The game vertex it walks from, `m_tPrevGraphID`.
  pub update_previous_game_vertex_id: u16,
  pub update_distance_from_point: f32,
  pub update_distance_to_point: f32,
}

impl AlifeObjectMonsterAbstract {
  /// Place the creature at `position`, facing `direction`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.base.move_to(position, direction);
  }
}

impl AlifeObjectMonsterAbstract {
  /// `m_task_reached`, a `bool` the bytes hold raw and fields as a `u16` of 0 or 1
  /// (`CSE_ALifeMonsterAbstract::STATE_Read`, `xrServer_Objects_ALife_Monsters.cpp:1126`).
  fn read_task_reached<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<u8> {
    if reader.is_field_form() {
      Ok(u8::from(reader.read_u16::<T>()? != 0))
    } else {
      reader.read_u8()
    }
  }

  /// As `CSE_ALifeMonsterAbstract::STATE_Write` writes it (`xrServer_Objects_ALife_Monsters.cpp:1105`).
  fn write_task_reached<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    if writer.is_field_form() {
      writer.write_u16::<T>(u16::from(self.task_reached != 0))
    } else {
      writer.write_u8(self.task_reached)
    }
  }
}

impl AlifePacketsReadWrite for AlifeObjectMonsterAbstract {
  /// The creature's, then the graph vertices it walks between as the constructor sets them: none, copied from the
  /// object's game vertex while that is still unset (`xrServer_Objects_ALife_Monsters.cpp:1017`), and no distance.
  fn set_spawned_update(&mut self, header: &SpawnPacketHeader) {
    self.base.set_spawned_update(header);
    self.update_next_game_vertex_id = u16::MAX;
    self.update_previous_game_vertex_id = u16::MAX;
    self.update_distance_from_point = 0.0;
    self.update_distance_to_point = 0.0;
  }

  /// Read the monster from both packets.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectCreature::read::<T, _, _>(spawn, update, layout)?,
      out_space_restrictors: spawn.read_w1251_string()?,
      in_space_restrictors: spawn.read_w1251_string()?,
      smart_terrain_id: spawn.read_u16::<T>()?,
      task_reached: Self::read_task_reached::<T, _>(spawn)?,
      update_next_game_vertex_id: update.read_u16::<T>()?,
      update_previous_game_vertex_id: update.read_u16::<T>()?,
      update_distance_from_point: update.read_f32::<T>()?,
      update_distance_to_point: update.read_f32::<T>()?,
    })
  }

  /// Write the monster into both packets.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult {
    self.base.write::<T, _, _>(spawn, update, layout)?;

    spawn.write_w1251_string(&self.out_space_restrictors)?;
    spawn.write_w1251_string(&self.in_space_restrictors)?;
    spawn.write_u16::<T>(self.smart_terrain_id)?;
    self.write_task_reached::<T, _>(spawn)?;

    update.write_u16::<T>(self.update_next_game_vertex_id)?;
    update.write_u16::<T>(self.update_previous_game_vertex_id)?;
    update.write_f32::<T>(self.update_distance_from_point)?;
    update.write_f32::<T>(self.update_distance_to_point)?;

    Ok(())
  }

  /// Import monster data from ltx config section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectCreature::import(section_name, ltx, layout)?,
      out_space_restrictors: section.read_field("monster.out_space_restrictors")?,
      in_space_restrictors: section.read_field("monster.in_space_restrictors")?,
      smart_terrain_id: section.read_field("monster.smart_terrain_id")?,
      task_reached: section.read_field("monster.task_reached")?,
      update_next_game_vertex_id: section.read_field("monster.update_next_game_vertex_id")?,
      update_previous_game_vertex_id: section.read_field("monster.update_previous_game_vertex_id")?,
      update_distance_from_point: section.read_field("monster.update_distance_from_point")?,
      update_distance_to_point: section.read_field("monster.update_distance_to_point")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx, layout)?;

    ltx
      .with_section(section_name)
      .set("monster.out_space_restrictors", &self.out_space_restrictors)
      .set("monster.in_space_restrictors", &self.in_space_restrictors)
      .set("monster.smart_terrain_id", self.smart_terrain_id)
      .set("monster.task_reached", self.task_reached)
      .set("monster.update_next_game_vertex_id", self.update_next_game_vertex_id)
      .set(
        "monster.update_previous_game_vertex_id",
        self.update_previous_game_vertex_id,
      )
      .set("monster.update_distance_from_point", self.update_distance_from_point)
      .set("monster.update_distance_to_point", self.update_distance_to_point);

    Ok(())
  }
}

#[cfg(any(test, feature = "mocks"))]
impl AlifeObjectMonsterAbstract {
  /// A creature at `position` as the SDK compiles one: no restrictions, no smart terrain, and its graph edge unset.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectCreature::new_mock(position),
      out_space_restrictors: String::new(),
      in_space_restrictors: String::new(),
      smart_terrain_id: u16::MAX,
      task_reached: 0,
      update_next_game_vertex_id: u16::MAX,
      update_previous_game_vertex_id: u16::MAX,
      update_distance_from_point: 0.0,
      update_distance_to_point: 0.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_monster_abstract::AlifeObjectMonsterAbstract;
  use crate::data::alife::spawn_layout::SpawnLayout;

  fn new_object() -> AlifeObjectMonsterAbstract {
    let mut object: AlifeObjectMonsterAbstract = AlifeObjectMonsterAbstract::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    object.out_space_restrictors = String::from("zat_a2_restrictor,zat_b5_restrictor");
    object.in_space_restrictors = String::from("zat_sr_noweap");
    object.smart_terrain_id = 812;
    object.task_reached = 1;
    object.update_next_game_vertex_id = 44;
    object.update_previous_game_vertex_id = 43;
    object.update_distance_from_point = 2.5;
    object.update_distance_to_point = 10.25;

    object
  }

  /// Two strings, the smart terrain and a byte after the creature's 88 spawn bytes; two vertices and two distances
  /// after its 40 update bytes.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectMonsterAbstract = new_object();

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (88 + 36 + 14 + 2 + 1, 40 + 12, original)
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectMonsterAbstract = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx, SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(
      AlifeObjectMonsterAbstract::import("data", &ltx, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectMonsterAbstract = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectMonsterAbstract>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
