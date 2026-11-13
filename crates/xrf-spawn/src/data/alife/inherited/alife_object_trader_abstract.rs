use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::spawn_layout::SpawnLayout;

/// `CSE_ALifeTraderAbstract`, the trading part of a trader, a human and the actor. Up to spawn version 124 the packet
/// stores neither of its corpse's access fields, which then hold the constructor's: open to taking, not closed
/// (`xrServer_Objects_ALife_Monsters.cpp:138-139`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectTraderAbstract {
  pub money: u32,
  pub specific_character: String,
  pub trader_flags: u32,
  pub character_profile: String,
  pub community_index: u32,
  pub rank: u32,
  pub reputation: u32,
  pub character_name: String,
  pub dead_body_can_take: u8,
  pub dead_body_closed: u8,
}

impl AlifeObjectTraderAbstract {
  /// `dead_body_can_take` where the packet does not store it.
  pub(crate) const DEFAULT_DEAD_BODY_CAN_TAKE: u8 = 1;

  /// `dead_body_closed` where the packet does not store it.
  pub(crate) const DEFAULT_DEAD_BODY_CLOSED: u8 = 0;

  /// Refuse corpse access fields the layout has no place for unless they hold the constructor's values.
  fn assert_corpse_access_fits(&self, layout: SpawnLayout) -> XrfResult {
    if layout.has_trader_corpse_access() {
      return Ok(());
    }

    layout.assert_absent_default(
      "dead_body_can_take",
      &self.dead_body_can_take,
      &Self::DEFAULT_DEAD_BODY_CAN_TAKE,
    )?;
    layout.assert_absent_default(
      "dead_body_closed",
      &self.dead_body_closed,
      &Self::DEFAULT_DEAD_BODY_CLOSED,
    )
  }
}

impl AlifeStateReadWrite for AlifeObjectTraderAbstract {
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    let mut trader: Self = Self {
      money: reader.read_u32::<T>()?,
      specific_character: reader.read_w1251_string()?,
      trader_flags: reader.read_u32::<T>()?,
      character_profile: reader.read_w1251_string()?,
      community_index: reader.read_u32::<T>()?,
      rank: reader.read_u32::<T>()?,
      reputation: reader.read_u32::<T>()?,
      character_name: reader.read_w1251_string()?,
      dead_body_can_take: Self::DEFAULT_DEAD_BODY_CAN_TAKE,
      dead_body_closed: Self::DEFAULT_DEAD_BODY_CLOSED,
    };

    if layout.has_trader_corpse_access() {
      trader.dead_body_can_take = reader.read_u8()?;
      trader.dead_body_closed = reader.read_u8()?;
    }

    Ok(trader)
  }

  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    self.assert_corpse_access_fits(layout)?;
    writer.write_u32::<T>(self.money)?;
    writer.write_w1251_string(&self.specific_character)?;
    writer.write_u32::<T>(self.trader_flags)?;
    writer.write_w1251_string(&self.character_profile)?;
    writer.write_u32::<T>(self.community_index)?;
    writer.write_u32::<T>(self.rank)?;
    writer.write_u32::<T>(self.reputation)?;
    writer.write_w1251_string(&self.character_name)?;

    if layout.has_trader_corpse_access() {
      writer.write_u8(self.dead_body_can_take)?;
      writer.write_u8(self.dead_body_closed)?;
    }

    Ok(())
  }

  fn import_state<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;
    let mut trader: Self = Self {
      money: section.read_field("trader.money")?,
      specific_character: section.read_field("trader.specific_character")?,
      trader_flags: section.read_field("trader.trader_flags")?,
      character_profile: section.read_field("trader.character_profile")?,
      community_index: section.read_field("trader.community_index")?,
      rank: section.read_field("trader.rank")?,
      reputation: section.read_field("trader.reputation")?,
      character_name: section.read_field("trader.character_name")?,
      dead_body_can_take: Self::DEFAULT_DEAD_BODY_CAN_TAKE,
      dead_body_closed: Self::DEFAULT_DEAD_BODY_CLOSED,
    };

    if layout.has_trader_corpse_access() {
      trader.dead_body_can_take = section.read_field("trader.dead_body_can_take")?;
      trader.dead_body_closed = section.read_field("trader.dead_body_closed")?;
    }

    Ok(trader)
  }

  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.assert_corpse_access_fits(layout)?;

    ltx
      .with_section(section_name)
      .set("trader.money", self.money)
      .set("trader.specific_character", &self.specific_character)
      .set("trader.trader_flags", self.trader_flags)
      .set("trader.character_profile", &self.character_profile)
      .set("trader.community_index", self.community_index)
      .set("trader.rank", self.rank)
      .set("trader.reputation", self.reputation)
      .set("trader.character_name", &self.character_name);

    if layout.has_trader_corpse_access() {
      ltx
        .with_section(section_name)
        .set("trader.dead_body_can_take", self.dead_body_can_take)
        .set("trader.dead_body_closed", self.dead_body_closed);
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
  use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
  use crate::data::alife::spawn_layout::SpawnLayout;

  fn new_trader(dead_body_can_take: u8, dead_body_closed: u8) -> AlifeObjectTraderAbstract {
    AlifeObjectTraderAbstract {
      money: 1453,
      specific_character: String::from("specific-character"),
      trader_flags: 33,
      character_profile: String::from("character-profile"),
      community_index: 4,
      rank: 211,
      reputation: 300,
      character_name: String::from("character-name"),
      dead_body_can_take,
      dead_body_closed,
    }
  }

  fn layout(version: u16) -> SpawnLayout {
    SpawnLayout {
      version,
      script_version: 8,
    }
  }

  /// The bytes written at `layout`, and the trader read back from them whole.
  fn read_back(
    trader: &AlifeObjectTraderAbstract,
    layout: SpawnLayout,
  ) -> XrfResult<(usize, AlifeObjectTraderAbstract)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    trader.write_state::<XRayByteOrder, _>(&mut writer, layout)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: AlifeObjectTraderAbstract =
      AlifeObjectTraderAbstract::read_state::<XRayByteOrder, _>(&mut reader, layout)?;

    reader.assert_read("Expect the trader to be read whole")?;

    Ok((writer.buffer.len(), read))
  }

  #[test]
  fn writes_corpse_access_above_version_124_only() -> XrfResult {
    let trader: AlifeObjectTraderAbstract = new_trader(1, 0);
    let (with_access, read_with) = read_back(&trader, layout(125))?;
    let (without_access, read_without) = read_back(&trader, layout(124))?;

    assert_eq!(with_access, 74);
    assert_eq!(without_access, with_access - 2);
    assert_eq!(read_with, trader);
    assert_eq!(read_without, trader);

    let trader: AlifeObjectTraderAbstract = new_trader(0, 1);

    assert_eq!(read_back(&trader, SpawnLayout::CALL_OF_PRIPYAT)?, (74, trader));

    Ok(())
  }

  #[test]
  fn refuses_to_drop_corpse_access_up_to_version_124() {
    for (trader, field) in [
      (new_trader(0, 0), "dead_body_can_take"),
      (new_trader(1, 1), "dead_body_closed"),
    ] {
      let error: String = trader
        .write_state::<XRayByteOrder, _>(&mut ChunkWriter::new(), layout(124))
        .expect_err("a corpse access field the packet cannot hold")
        .to_string();

      assert!(
        error.starts_with(&format!("Invalid error: Field '{field}' holds")),
        "{error}"
      );
      assert!(
        trader.export_state("data", &mut Ltx::new(), layout(124)).is_err(),
        "{field} in text"
      );
    }
  }

  #[test]
  fn test_import_export() -> XrfResult {
    for (trader, version) in [(new_trader(0, 1), 128), (new_trader(1, 0), 124)] {
      let mut ltx: Ltx = Ltx::new();

      trader.export_state("data", &mut ltx, layout(version))?;

      for key in ["trader.dead_body_can_take", "trader.dead_body_closed"] {
        assert_eq!(ltx.get_from("data", key).is_some(), version == 128, "{key} {version}");
      }

      assert_eq!(
        AlifeObjectTraderAbstract::import_state("data", &ltx, layout(version))?,
        trader
      );
    }

    Ok(())
  }
}
