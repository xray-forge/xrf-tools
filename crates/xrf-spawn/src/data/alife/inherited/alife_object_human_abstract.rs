use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_math::Vector3d;
use xrf_utils::{narrow_format_size, vector_from_string, vector_to_string};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_object_monster_abstract::AlifeObjectMonsterAbstract;
use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;

/// A human, `CSE_ALifeHumanAbstract`: a trader and a monster, in that order, so unlike the actor's its spawn packet
/// opens with the trader's fields, ahead of every field the object inherits. The human brain follows them with the
/// equipment and weapon preferences it picks at random. Its update packet is the monster's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectHumanAbstract {
  pub trader: AlifeObjectTraderAbstract,
  pub base: AlifeObjectMonsterAbstract,
  /// `CALifeHumanBrain::m_cpEquipmentPreferences`, five values in 0..3 as the SDK writes them.
  pub equipment_preferences: Vec<u8>,
  /// `CALifeHumanBrain::m_cpMainWeaponPreferences`, four values in 0..3 as the SDK writes them.
  pub main_weapon_preferences: Vec<u8>,
}

impl AlifeObjectHumanAbstract {
  /// Place the human at `position`, facing `direction`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.base.move_to(position, direction);
  }

  /// Read a preference list as `load_data` reads an `xr_vector<u8>`: a `u32` count, then a byte each. The brain reads
  /// it from bytes alone (`CALifeHumanBrain::on_state_read`): fields hold none, and the editor keeps the values its
  /// constructor drew at random, which here read as none.
  fn read_preferences<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Vec<u8>> {
    if reader.is_field_form() {
      return Ok(Vec::new());
    }

    let count: u32 = reader.read_u32::<T>()?;

    reader.read_bytes(count as usize)
  }

  /// Write a preference list as `save_data` writes it, to bytes alone (`CALifeHumanBrain::on_state_write`).
  fn write_preferences<T: ByteOrder, W: ChunkFieldWriter>(writer: &mut W, preferences: &[u8]) -> XrfResult {
    if writer.is_field_form() {
      return Ok(());
    }

    writer.write_u32::<T>(narrow_format_size(preferences.len(), "human preferences")?)?;
    writer.write_bytes(preferences)
  }
}

impl AlifePacketsReadWrite for AlifeObjectHumanAbstract {
  fn set_spawned_update(&mut self, header: &SpawnPacketHeader) {
    self.base.set_spawned_update(header);
  }

  /// Read the human from both packets.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self> {
    Ok(Self {
      trader: AlifeObjectTraderAbstract::read_state::<T, _>(spawn, layout)?,
      base: AlifeObjectMonsterAbstract::read::<T, _, _>(spawn, update, layout)?,
      equipment_preferences: Self::read_preferences::<T, _>(spawn)?,
      main_weapon_preferences: Self::read_preferences::<T, _>(spawn)?,
    })
  }

  /// Write the human into both packets.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult {
    self.trader.write_state::<T, _>(spawn, layout)?;

    self.base.write::<T, _, _>(spawn, update, layout)?;

    Self::write_preferences::<T, _>(spawn, &self.equipment_preferences)?;
    Self::write_preferences::<T, _>(spawn, &self.main_weapon_preferences)
  }

  /// Import human data from ltx config section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      trader: AlifeObjectTraderAbstract::import_state(section_name, ltx, layout)?,
      base: AlifeObjectMonsterAbstract::import(section_name, ltx, layout)?,
      equipment_preferences: vector_from_string(&section.read_field::<String>("human.equipment_preferences")?)?,
      main_weapon_preferences: vector_from_string(&section.read_field::<String>("human.main_weapon_preferences")?)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.trader.export_state(section_name, ltx, layout)?;
    self.base.export(section_name, ltx, layout)?;

    ltx
      .with_section(section_name)
      .set(
        "human.equipment_preferences",
        vector_to_string(&self.equipment_preferences),
      )
      .set(
        "human.main_weapon_preferences",
        vector_to_string(&self.main_weapon_preferences),
      );

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectHumanAbstract {
  /// A human at `position` as the SDK compiles one: a character profile, no money, and random preferences.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      trader: AlifeObjectTraderAbstract {
        money: 0,
        specific_character: String::new(),
        trader_flags: 1,
        character_profile: String::from("sim_default_stalker_0"),
        community_index: u32::MAX,
        rank: 0x8000_0001,
        reputation: 0x8000_0001,
        character_name: String::new(),
        dead_body_can_take: 1,
        dead_body_closed: 0,
      },
      base: AlifeObjectMonsterAbstract::new_mock(position),
      equipment_preferences: vec![1, 0, 2, 2, 1],
      main_weapon_preferences: vec![0, 2, 1, 1],
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_human_abstract::AlifeObjectHumanAbstract;
  use crate::data::alife::inherited::alife_object_monster_abstract::AlifeObjectMonsterAbstract;
  use crate::data::alife::spawn_layout::SpawnLayout;

  /// The mock trader's money, two strings, flags, profile, three ranks and two bytes.
  const TRADER_SIZE: usize = 4 + 1 + 4 + "sim_default_stalker_0".len() + 1 + 12 + 1 + 2;

  /// The trader's fields come first, then the monster's, then two counted lists; the update packet is the monster's.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectHumanAbstract = AlifeObjectHumanAbstract::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let (monster_size, _, _) = original.base.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (TRADER_SIZE + monster_size + 4 + 5 + 4 + 4, 52, original)
    );

    Ok(())
  }

  /// The trader's money opens the packet where the actor's would hold its game vertex.
  #[test]
  fn opens_its_spawn_fields_with_the_trader() -> XrfResult {
    let mut original: AlifeObjectHumanAbstract = AlifeObjectHumanAbstract::new_mock(Vector3d::ZERO);
    let mut spawn: ChunkWriter = ChunkWriter::new();
    let mut update: ChunkWriter = ChunkWriter::new();

    original.trader.money = 0x0102_0304;
    original.write::<XRayByteOrder, _, _>(&mut spawn, &mut update, SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(&spawn.buffer[..4], &[4, 3, 2, 1]);

    let mut monster: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer[TRADER_SIZE..])?;
    let mut monster_update: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update.buffer)?;

    assert_eq!(
      AlifeObjectMonsterAbstract::read::<XRayByteOrder, _, _>(
        &mut monster,
        &mut monster_update,
        SpawnLayout::CALL_OF_PRIPYAT
      )?,
      original.base
    );

    Ok(())
  }

  /// Up to spawn version 124 the trader stores no corpse access, so the monster's fields start two bytes earlier.
  #[test]
  fn reads_its_trader_without_corpse_access_up_to_version_124() -> XrfResult {
    let original: AlifeObjectHumanAbstract = AlifeObjectHumanAbstract::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let layout = |version: u16| SpawnLayout {
      version,
      script_version: 8,
    };
    let (with_access, update_with, read_with) = original.read_back::<XRayByteOrder>(layout(125))?;
    let (without_access, update_without, read_without) = original.read_back::<XRayByteOrder>(layout(124))?;

    assert_eq!(without_access, with_access - 2);
    assert_eq!(update_without, update_with);
    assert_eq!(read_with, original);
    assert_eq!(read_without, original);

    let mut closed: AlifeObjectHumanAbstract = original;

    closed.trader.dead_body_closed = 1;

    assert!(closed.read_back::<XRayByteOrder>(layout(124)).is_err());
    assert!(closed.export("data", &mut Ltx::new(), layout(124)).is_err());

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectHumanAbstract = AlifeObjectHumanAbstract::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx, SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(
      AlifeObjectHumanAbstract::import("data", &ltx, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectHumanAbstract = AlifeObjectHumanAbstract::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(
      serde_json::from_str::<AlifeObjectHumanAbstract>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
