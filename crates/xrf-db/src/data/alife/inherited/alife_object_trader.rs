use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::Ltx;

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
use crate::export::LtxImportExport;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectTrader {
  pub base: AlifeObjectDynamicVisual,
  pub trader: AlifeObjectTraderAbstract,
}

impl ChunkReadWrite for AlifeObjectTrader {
  /// Read trader data from the chunk.
  ///
  /// `CSE_ALifeTrader::STATE_Read` follows its two bases with three blocks gated below version 118,
  /// none of which the supported versions reach.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      trader: reader.read_xr::<T, _>()?,
    })
  }

  /// Write trader data into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr::<T, _>(&self.trader)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectTrader {
  /// Import trader object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      trader: AlifeObjectTraderAbstract::import(section_name, ltx)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.trader.export(section_name, ltx)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_trader::AlifeObjectTrader;
  use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
  use crate::export::LtxImportExport;

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
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");
    let original: AlifeObjectTrader = new_mock();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 126);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 126);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 126 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectTrader::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();
    let original: AlifeObjectTrader = new_mock();

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectTrader::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectTrader = new_mock();

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serde_json::from_str::<AlifeObjectTrader>(&serialized)?, original);

    Ok(())
  }
}
