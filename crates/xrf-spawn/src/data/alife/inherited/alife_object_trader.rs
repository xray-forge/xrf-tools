use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;
use xrf_ltx::{LtxImportExport, LtxRead, LtxWrite};

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
use crate::data::alife::spawn_layout::SpawnLayout;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectTrader {
  pub base: AlifeObjectDynamicVisual,
  pub trader: AlifeObjectTraderAbstract,
}

impl AlifeStateReadWrite for AlifeObjectTrader {
  /// Read trader data from the chunk.
  ///
  /// `CSE_ALifeTrader::STATE_Read` follows its two bases with three blocks gated below version 118,
  /// none of which the supported versions reach.
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      trader: AlifeObjectTraderAbstract::read_state::<T, _>(reader, layout)?,
    })
  }

  /// Write trader data into the chunk.
  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    self.trader.write_state::<T, _>(writer, layout)
  }

  /// Import trader object data from ltx config section.
  fn import_state<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      trader: AlifeObjectTraderAbstract::import_state(section_name, ltx, layout)?,
    })
  }

  /// Export object data into ltx file.
  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.trader.export_state(section_name, ltx, layout)
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_test_utils::GeneratedResources;

  use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_trader::AlifeObjectTrader;
  use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
  use crate::data::alife::spawn_layout::SpawnLayout;

  /// Shaped like the version 128 traders in the Call of Chernobyl and Anomaly spawns, whose rank and
  /// reputation are stored with the high bit set.
  fn new_mock() -> AlifeObjectTrader {
    AlifeObjectTrader {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 253,
          distance: 25.53,
          direct_control: 236,
          level_vertex_id: 26,
          flags: 364,
          custom_data: String::from("custom-data"),
          story_id: 26,
          spawn_story_id: 346,
        },
        visual_name: String::from("actors\\stalker_trader\\stalker_trader_1"),
        visual_flags: 0,
      },
      trader: AlifeObjectTraderAbstract {
        money: 1_000_000,
        specific_character: String::new(),
        trader_flags: 0,
        character_profile: String::from("esc_2_12_stalker_trader"),
        community_index: 4_294_967_295,
        rank: 2_147_483_649,
        reputation: 2_147_483_649,
        character_name: String::new(),
        dead_body_can_take: 1,
        dead_body_closed: 0,
      },
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let original: AlifeObjectTrader = new_mock();

    original.write_state::<XRayByteOrder, _>(&mut writer, SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(writer.bytes_written(), 126);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      AlifeObjectTrader::read_state::<XRayByteOrder, _>(&mut reader, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );
    reader.assert_read("Expect the trader to be read whole")?;

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();
    let original: AlifeObjectTrader = new_mock();

    original.export_state("data", &mut ltx, SpawnLayout::CALL_OF_PRIPYAT)?;

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

    assert_eq!(
      AlifeObjectTrader::import_state("data", &source, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectTrader = new_mock();

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serde_json::from_str::<AlifeObjectTrader>(&serialized)?, original);

    Ok(())
  }
}
