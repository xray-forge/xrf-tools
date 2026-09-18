use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl ChunkReadWrite for AlifeObjectTraderAbstract {
  /// Read trader data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      money: reader.read_u32::<T>()?,
      specific_character: reader.read_w1251_string()?,
      trader_flags: reader.read_u32::<T>()?,
      character_profile: reader.read_w1251_string()?,
      community_index: reader.read_u32::<T>()?,
      rank: reader.read_u32::<T>()?,
      reputation: reader.read_u32::<T>()?,
      character_name: reader.read_w1251_string()?,
      dead_body_can_take: reader.read_u8()?,
      dead_body_closed: reader.read_u8()?,
    })
  }

  /// Write trader data into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.money)?;
    writer.write_w1251_string(&self.specific_character)?;
    writer.write_u32::<T>(self.trader_flags)?;
    writer.write_w1251_string(&self.character_profile)?;
    writer.write_u32::<T>(self.community_index)?;
    writer.write_u32::<T>(self.rank)?;
    writer.write_u32::<T>(self.reputation)?;
    writer.write_w1251_string(&self.character_name)?;
    writer.write_u8(self.dead_body_can_take)?;
    writer.write_u8(self.dead_body_closed)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectTraderAbstract {
  /// Import trader data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      money: read_ltx_field("trader.money", section)?,
      specific_character: read_ltx_field("trader.specific_character", section)?,
      trader_flags: read_ltx_field("trader.trader_flags", section)?,
      character_profile: read_ltx_field("trader.character_profile", section)?,
      community_index: read_ltx_field("trader.community_index", section)?,
      rank: read_ltx_field("trader.rank", section)?,
      reputation: read_ltx_field("trader.reputation", section)?,
      character_name: read_ltx_field("trader.character_name", section)?,
      dead_body_can_take: read_ltx_field("trader.dead_body_can_take", section)?,
      dead_body_closed: read_ltx_field("trader.dead_body_closed", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("trader.money", self.money.to_string())
      .set("trader.specific_character", &self.specific_character)
      .set("trader.trader_flags", self.trader_flags.to_string())
      .set("trader.character_profile", &self.character_profile)
      .set("trader.community_index", self.community_index.to_string())
      .set("trader.rank", self.rank.to_string())
      .set("trader.reputation", self.reputation.to_string())
      .set("trader.character_name", &self.character_name)
      .set("trader.dead_body_can_take", self.dead_body_can_take.to_string())
      .set("trader.dead_body_closed", self.dead_body_closed.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectTraderAbstract = AlifeObjectTraderAbstract {
      money: 1453,
      specific_character: String::from("specific-character"),
      trader_flags: 33,
      character_profile: String::from("character-profile"),
      community_index: 4,
      rank: 211,
      reputation: 300,
      character_name: String::from("character-name"),
      dead_body_can_take: 1,
      dead_body_closed: 0,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 74);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 74);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 74 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(
      AlifeObjectTraderAbstract::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }
}
