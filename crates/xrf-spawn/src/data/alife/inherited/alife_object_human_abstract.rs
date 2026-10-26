use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;
use xrf_utils::{to_format_size, vector_from_string, vector_to_string};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_monster_abstract::AlifeObjectMonsterAbstract;
use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;

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

  /// Read a preference list as `load_data` reads an `xr_vector<u8>`: a `u32` count, then a byte each.
  fn read_preferences<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<u8>> {
    let count: u32 = reader.read_u32::<T>()?;

    reader.read_bytes(count as usize)
  }

  fn write_preferences<T: ByteOrder>(writer: &mut ChunkWriter, preferences: &[u8]) -> XrfResult {
    writer.write_u32::<T>(to_format_size(preferences.len(), "human preferences")?)?;
    writer.buffer.extend_from_slice(preferences);

    Ok(())
  }
}

impl AlifePacketsReadWrite for AlifeObjectHumanAbstract {
  /// Read the human from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      trader: spawn.read_xr::<T, _>()?,
      base: AlifeObjectMonsterAbstract::read::<T, _>(spawn, update)?,
      equipment_preferences: Self::read_preferences::<T, _>(spawn)?,
      main_weapon_preferences: Self::read_preferences::<T, _>(spawn)?,
    })
  }

  /// Write the human into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    spawn.write_xr::<T, _>(&self.trader)?;

    self.base.write::<T>(spawn, update)?;

    Self::write_preferences::<T>(spawn, &self.equipment_preferences)?;
    Self::write_preferences::<T>(spawn, &self.main_weapon_preferences)
  }
}

impl LtxImportExport for AlifeObjectHumanAbstract {
  /// Import human data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      trader: AlifeObjectTraderAbstract::import(section_name, ltx)?,
      base: AlifeObjectMonsterAbstract::import(section_name, ltx)?,
      equipment_preferences: vector_from_string(&section.read_field::<String>("human.equipment_preferences")?)?,
      main_weapon_preferences: vector_from_string(&section.read_field::<String>("human.main_weapon_preferences")?)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.trader.export(section_name, ltx)?;
    self.base.export(section_name, ltx)?;

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
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_human_abstract::AlifeObjectHumanAbstract;
  use crate::data::alife::inherited::alife_object_monster_abstract::AlifeObjectMonsterAbstract;

  /// The mock trader's money, two strings, flags, profile, three ranks and two bytes.
  const TRADER_SIZE: usize = 4 + 1 + 4 + "sim_default_stalker_0".len() + 1 + 12 + 1 + 2;

  /// The trader's fields come first, then the monster's, then two counted lists; the update packet is the monster's.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectHumanAbstract = AlifeObjectHumanAbstract::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let (monster_size, _, _) = original.base.read_back::<XRayByteOrder>()?;

    assert_eq!(
      original.read_back::<XRayByteOrder>()?,
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
    original.write::<XRayByteOrder>(&mut spawn, &mut update)?;

    assert_eq!(&spawn.buffer[..4], &[4, 3, 2, 1]);

    let mut monster: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer[TRADER_SIZE..])?;
    let mut monster_update: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update.buffer)?;

    assert_eq!(
      AlifeObjectMonsterAbstract::read::<XRayByteOrder, _>(&mut monster, &mut monster_update)?,
      original.base
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectHumanAbstract = AlifeObjectHumanAbstract::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectHumanAbstract::import("data", &ltx)?, original);

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
