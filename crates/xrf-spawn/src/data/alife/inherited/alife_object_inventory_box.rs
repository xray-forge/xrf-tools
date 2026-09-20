use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section, read_ltx_field};

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectInventoryBox {
  pub base: AlifeObjectDynamicVisual,
  pub can_take: u8,
  pub is_closed: u8,
  pub tip: String,
}

impl ChunkReadWrite for AlifeObjectInventoryBox {
  /// Read inventory object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      can_take: reader.read_u8()?,
      is_closed: reader.read_u8()?,
      tip: reader.read_w1251_string()?,
    })
  }

  /// Write inventory object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_u8(self.can_take)?;
    writer.write_u8(self.is_closed)?;
    writer.write_w1251_string(&self.tip)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectInventoryBox {
  /// Import ALife inventory box object from ltx config section.
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
      can_take: read_ltx_field("inventory_box.can_take", section)?,
      is_closed: read_ltx_field("inventory_box.is_closed", section)?,
      tip: read_ltx_field("inventory_box.tip", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("inventory_box.can_take", self.can_take.to_string())
      .set("inventory_box.is_closed", self.is_closed.to_string())
      .set("inventory_box.tip", &self.tip);

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
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_inventory_box::AlifeObjectInventoryBox;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectInventoryBox = AlifeObjectInventoryBox {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 2463,
          distance: 12.0,
          direct_control: 5634,
          level_vertex_id: 2533,
          flags: 64,
          custom_data: String::from("custom-data"),
          story_id: 2136,
          spawn_story_id: 0,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 0,
      },
      can_take: 0,
      is_closed: 1,
      tip: String::from("some-tip"),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 62);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 62);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 62 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(
      AlifeObjectInventoryBox::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectInventoryBox = AlifeObjectInventoryBox {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 236,
          distance: 15.0,
          direct_control: 15,
          level_vertex_id: 16,
          flags: 31,
          custom_data: String::from("custom-data"),
          story_id: 152,
          spawn_story_id: 20,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 20,
      },
      can_take: 1,
      is_closed: 0,
      tip: String::from("some-tip"),
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectInventoryBox::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectInventoryBox = AlifeObjectInventoryBox {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 25,
          distance: 1.0,
          direct_control: 25,
          level_vertex_id: 62,
          flags: 64,
          custom_data: String::from("custom-data"),
          story_id: 15,
          spawn_story_id: 0,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 23,
      },
      can_take: 1,
      is_closed: 1,
      tip: String::from("some-tip"),
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectInventoryBox>(&serialized)?, original);

    Ok(())
  }
}
