use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section, read_ltx_field};

use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectDynamicVisual {
  pub base: AlifeObjectAbstract,
  pub visual_name: String,
  pub visual_flags: u8,
}

impl ChunkReadWrite for AlifeObjectDynamicVisual {
  /// Read visual object data from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      visual_name: reader.read_w1251_string()?,
      visual_flags: reader.read_u8()?,
    })
  }

  /// Write visual ALife object data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_w1251_string(&self.visual_name)?;
    writer.write_u8(self.visual_flags)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectDynamicVisual {
  /// Import visual object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectAbstract::import(section_name, ltx)?,
      visual_name: read_ltx_field("dynamic_visual.visual_name", section)?,
      visual_flags: read_ltx_field("dynamic_visual.visual_flags", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("dynamic_visual.visual_name", &self.visual_name)
      .set("dynamic_visual.visual_flags", self.visual_flags.to_string());

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

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectDynamicVisual = AlifeObjectDynamicVisual {
      base: AlifeObjectAbstract {
        game_vertex_id: 1001,
        distance: 65.25,
        direct_control: 412421,
        level_vertex_id: 66231,
        flags: 33,
        custom_data: String::from("custom_data"),
        story_id: 400,
        spawn_story_id: 25,
      },
      visual_name: String::from("visual-name"),
      visual_flags: 33,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 51);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 51);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 51 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;
    let read_object: AlifeObjectDynamicVisual = AlifeObjectDynamicVisual::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read_object, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let first: AlifeObjectDynamicVisual = AlifeObjectDynamicVisual {
      base: AlifeObjectAbstract {
        game_vertex_id: 41243,
        distance: 24.5,
        direct_control: 523,
        level_vertex_id: 7243,
        flags: 23,
        custom_data: String::from("custom_data_first"),
        story_id: 253,
        spawn_story_id: 6262,
      },
      visual_name: String::from("visual-name-first"),
      visual_flags: 34,
    };

    let second: AlifeObjectDynamicVisual = AlifeObjectDynamicVisual {
      base: AlifeObjectAbstract {
        game_vertex_id: 325,
        distance: 12.65,
        direct_control: 143,
        level_vertex_id: 36421,
        flags: 342,
        custom_data: String::from("second"),
        story_id: 235,
        spawn_story_id: 45672,
      },
      visual_name: String::from("visual-name-second"),
      visual_flags: 54,
    };

    first.export("first", &mut ltx)?;
    second.export("second", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectDynamicVisual::import("first", &source)?, first);
    assert_eq!(AlifeObjectDynamicVisual::import("second", &source)?, second);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectDynamicVisual = AlifeObjectDynamicVisual {
      base: AlifeObjectAbstract {
        game_vertex_id: 325,
        distance: 523.100,
        direct_control: 25313,
        level_vertex_id: 235,
        flags: 342,
        custom_data: String::from("custom_data_serde"),
        story_id: 235,
        spawn_story_id: 45672,
      },
      visual_name: String::from("visual-name-serde"),
      visual_flags: 33,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<AlifeObjectDynamicVisual>(&serialized)?);

    Ok(())
  }
}
