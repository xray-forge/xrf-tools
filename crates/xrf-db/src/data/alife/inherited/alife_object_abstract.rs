use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::{decode_string_from_base64, encode_string_to_base64};

use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

/// Generic ALife object abstraction data.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectAbstract {
  pub game_vertex_id: u16,
  pub distance: f32,
  pub direct_control: u32,
  pub level_vertex_id: u32,
  pub flags: u32,
  pub custom_data: String,
  pub story_id: u32,
  pub spawn_story_id: u32,
}

impl ChunkReadWrite for AlifeObjectAbstract {
  /// Read generic ALife object base data from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      game_vertex_id: reader.read_u16::<T>()?,
      distance: reader.read_f32::<T>()?,
      direct_control: reader.read_u32::<T>()?,
      level_vertex_id: reader.read_u32::<T>()?,
      flags: reader.read_u32::<T>()?,
      custom_data: reader.read_w1251_string()?,
      story_id: reader.read_u32::<T>()?,
      spawn_story_id: reader.read_u32::<T>()?,
    })
  }

  /// Write abstract object data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.game_vertex_id)?;
    writer.write_f32::<T>(self.distance)?;
    writer.write_u32::<T>(self.direct_control)?;
    writer.write_u32::<T>(self.level_vertex_id)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_w1251_string(&self.custom_data)?;
    writer.write_u32::<T>(self.story_id)?;
    writer.write_u32::<T>(self.spawn_story_id)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectAbstract {
  /// Import generic ALife object base data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      game_vertex_id: read_ltx_field("abstract.game_vertex_id", section)?,
      distance: read_ltx_field("abstract.distance", section)?,
      direct_control: read_ltx_field("abstract.direct_control", section)?,
      level_vertex_id: read_ltx_field("abstract.level_vertex_id", section)?,
      flags: read_ltx_field("abstract.flags", section)?,
      custom_data: decode_string_from_base64(&read_ltx_field::<String>("abstract.custom_data", section)?)?,
      story_id: read_ltx_field("abstract.story_id", section)?,
      spawn_story_id: read_ltx_field("abstract.spawn_story_id", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("abstract.game_vertex_id", self.game_vertex_id.to_string())
      .set("abstract.distance", self.distance.to_string())
      .set("abstract.direct_control", self.direct_control.to_string())
      .set("abstract.level_vertex_id", self.level_vertex_id.to_string())
      .set("abstract.flags", self.flags.to_string())
      .set("abstract.custom_data", encode_string_to_base64(&self.custom_data))
      .set("abstract.story_id", self.story_id.to_string())
      .set("abstract.spawn_story_id", self.spawn_story_id.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::{XrfError, XrfResult};
  use xrf_ltx::Ltx;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectAbstract = AlifeObjectAbstract {
      game_vertex_id: 1001,
      distance: 65.25,
      direct_control: 412421,
      level_vertex_id: 66231,
      flags: 33,
      custom_data: String::from("custom_data"),
      story_id: 400,
      spawn_story_id: 25,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 38);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 38);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 38 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectAbstract::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let first: AlifeObjectAbstract = AlifeObjectAbstract {
      game_vertex_id: 1001,
      distance: 65.25,
      direct_control: 412421,
      level_vertex_id: 66231,
      flags: 33,
      custom_data: String::from("[custom_data_section] field1 = 1\r\n field2 = 2\r\n"),
      story_id: 400,
      spawn_story_id: 25,
    };

    let second: AlifeObjectAbstract = AlifeObjectAbstract {
      game_vertex_id: 1002,
      distance: 23.376,
      direct_control: 421,
      level_vertex_id: 75486,
      flags: 6,
      custom_data: String::from(""),
      story_id: 2345,
      spawn_story_id: 255,
    };

    first.export("first", &mut ltx)?;
    second.export("second", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectAbstract::import("first", &source)?, first);
    assert_eq!(AlifeObjectAbstract::import("second", &source)?, second);

    Ok(())
  }

  #[test]
  fn test_import_rejects_non_utf8_custom_data() -> XrfResult {
    let original: AlifeObjectAbstract = AlifeObjectAbstract {
      game_vertex_id: 1001,
      distance: 65.25,
      direct_control: 412421,
      level_vertex_id: 66231,
      flags: 33,
      custom_data: String::from("custom_data"),
      story_id: 400,
      spawn_story_id: 25,
    };
    let mut ltx: Ltx = Ltx::new();

    original.export("object", &mut ltx)?;
    ltx.with_section("object").set("abstract.custom_data", "_w");

    assert!(matches!(
      AlifeObjectAbstract::import("object", &ltx),
      Err(XrfError::Parsing { message }) if message.contains("not valid UTF-8")
    ));

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectAbstract = AlifeObjectAbstract {
      game_vertex_id: 1005,
      distance: 23.25,
      direct_control: 262342,
      level_vertex_id: 25341,
      flags: 34,
      custom_data: String::from("[custom_data_section] field1 = 1\r\n field2 = 2\r\n"),
      story_id: 433,
      spawn_story_id: 36,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<AlifeObjectAbstract>(&serialized)?);

    Ok(())
  }
}
