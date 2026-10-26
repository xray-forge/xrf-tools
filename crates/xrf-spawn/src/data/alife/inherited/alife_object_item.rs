use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_utils::{to_format_size, vector_from_string, vector_to_string};

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItem {
  pub base: AlifeObjectDynamicVisual,
  pub condition: f32,
  /// The upgrade sections installed on the item (`CSE_ALifeInventoryItem::m_upgrades`).
  pub upgrades: Vec<String>,
}

impl ChunkReadWrite for AlifeObjectItem {
  /// Read ALife item object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let base: AlifeObjectDynamicVisual = reader.read_xr::<T, _>()?;
    let condition: f32 = reader.read_f32::<T>()?;
    let count: u32 = reader.read_u32::<T>()?;
    // An upgrade is at least the terminator of an empty name.
    let mut upgrades: Vec<String> = reader.new_bounded_vec(count.into(), 1, "item upgrades")?;

    for _ in 0..count {
      upgrades.push(reader.read_w1251_string()?);
    }

    Ok(Self {
      base,
      condition,
      upgrades,
    })
  }

  /// Write item data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_f32::<T>(self.condition)?;
    writer.write_u32::<T>(to_format_size(self.upgrades.len(), "item upgrades")?)?;

    for upgrade in &self.upgrades {
      writer.write_w1251_string(upgrade)?;
    }

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItem {
  /// Import ALife item object data from ltx config section.
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
      condition: section.read_field("item.condition")?,
      upgrades: vector_from_string(&section.read_field::<String>("item.upgrades")?)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item.condition", self.condition.to_string())
      .set("item.upgrades", vector_to_string(&self.upgrades));

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write.chunk");

    let original: AlifeObjectItem = AlifeObjectItem {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 1002,
          distance: 65.25,
          direct_control: 41262,
          level_vertex_id: 618923,
          flags: 32,
          custom_data: String::from("custom_data"),
          story_id: 500,
          spawn_story_id: 35,
        },
        visual_name: String::from("abcd"),
        visual_flags: 33,
      },
      condition: 0.5,
      upgrades: Vec::new(),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 52);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 52);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 52 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectItem::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  /// An item placed upgraded carries each upgrade's section after the count, `save_data` of `m_upgrades`.
  #[test]
  fn reads_and_writes_the_upgrades_an_item_carries() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let original: AlifeObjectItem = AlifeObjectItem {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 1,
          distance: 0.0,
          direct_control: 1,
          level_vertex_id: 2,
          flags: 0,
          custom_data: String::new(),
          story_id: AlifeObjectAbstract::INVALID_STORY_ID,
          spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
        },
        visual_name: String::new(),
        visual_flags: 0,
      },
      condition: 1.0,
      upgrades: vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_vec(writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectItem::read::<XRayByteOrder, _>(&mut reader)?, original);
    reader.assert_read("Expect the upgrades to end the item")?;

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectItem = AlifeObjectItem {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 52,
          distance: 6.25,
          direct_control: 36,
          level_vertex_id: 3246,
          flags: 15,
          custom_data: String::from("custom_data"),
          story_id: 15,
          spawn_story_id: 734,
        },
        visual_name: String::from("abcd"),
        visual_flags: 234,
      },
      condition: 0.1,
      upgrades: vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")],
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

    assert_eq!(AlifeObjectItem::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItem = AlifeObjectItem {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 15,
          distance: 516.25,
          direct_control: 235,
          level_vertex_id: 737,
          flags: 45,
          custom_data: String::from("custom_data"),
          story_id: 364,
          spawn_story_id: 44,
        },
        visual_name: String::from("abcd"),
        visual_flags: 25,
      },
      condition: 0.65,
      upgrades: vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")],
    };

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectItem>(&serialized)?, original);

    Ok(())
  }
}
