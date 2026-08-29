use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::constants::FLAG_SKELETON_SAVED_DATA;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectSkeleton {
  pub name: String,
  pub flags: u8,
  pub source_id: u16,
}

impl AlifeObjectSkeleton {
  /// Reject a skeleton that promises saved bone data.
  ///
  /// The engine appends `CSE_PHSkeleton::data_load` bones after the header when the flag is set
  /// (`xray-16/src/xrServerEntities/xrServer_Objects.cpp:202`). This type cannot hold that payload, so
  /// reading it would drop bytes and writing it would declare bones the chunk does not carry.
  // todo: Read and write the saved bone payload described by `CSE_PHSkeleton::data_load`.
  fn assert_no_saved_bone_data(&self) -> XrfResult {
    if self.flags & FLAG_SKELETON_SAVED_DATA == 0 {
      return Ok(());
    }

    Err(XrfError::new_parsing_error(format!(
      "Skeleton '{}' declares saved bone data (flags {}), saved skeleton bones are not implemented",
      self.name, self.flags
    )))
  }
}

impl ChunkReadWrite for AlifeObjectSkeleton {
  /// Read skeleton data from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let object = Self {
      name: reader.read_w1251_string()?,
      flags: reader.read_u8()?,
      source_id: reader.read_u16::<XRayByteOrder>()?,
    };

    object.assert_no_saved_bone_data()?;

    Ok(object)
  }

  /// Write skeleton data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.assert_no_saved_bone_data()?;

    writer.write_w1251_string(&self.name)?;
    writer.write_u8(self.flags)?;
    writer.write_u16::<XRayByteOrder>(self.source_id)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectSkeleton {
  /// Import skeleton data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let object = Self {
      name: read_ltx_field("skeleton.name", section)?,
      flags: read_ltx_field("skeleton.flags", section)?,
      source_id: read_ltx_field("skeleton.source_id", section)?,
    };

    object.assert_no_saved_bone_data()?;

    Ok(object)
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("skeleton.name", &self.name)
      .set("skeleton.flags", self.flags.to_string())
      .set("skeleton.source_id", self.source_id.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};

  use byteorder::WriteBytesExt;
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

  use crate::constants::FLAG_SKELETON_SAVED_DATA;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectSkeleton = AlifeObjectSkeleton {
      name: String::from("test-name"),
      flags: 33,
      source_id: 753,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 13);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 13);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 13 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;
    let read_object: AlifeObjectSkeleton = AlifeObjectSkeleton::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read_object, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let first: AlifeObjectSkeleton = AlifeObjectSkeleton {
      name: String::from("test-name-first"),
      flags: 33,
      source_id: 753,
    };

    let second: AlifeObjectSkeleton = AlifeObjectSkeleton {
      name: String::from("test-name-second"),
      flags: 50,
      source_id: 526,
    };

    first.export("first", &mut ltx)?;
    second.export("second", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectSkeleton::import("first", &source)?, first);
    assert_eq!(AlifeObjectSkeleton::import("second", &source)?, second);

    Ok(())
  }

  #[test]
  fn test_read_rejects_saved_bone_data() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_saved_bone_data.chunk");

    writer.write_w1251_string("test-name")?;
    writer.write_u8(FLAG_SKELETON_SAVED_DATA)?;
    writer.write_u16::<XRayByteOrder>(753)?;

    writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    let mut reader: ChunkReader =
      ChunkReader::from_slice(open_generated_test_resource_as_slice(&filename)?)?.read_child_by_index(0)?;

    assert!(matches!(
      AlifeObjectSkeleton::read::<XRayByteOrder, _>(&mut reader),
      Err(XrfError::Parsing { message }) if message.contains("saved skeleton bones are not implemented")
    ));

    Ok(())
  }

  #[test]
  fn test_write_rejects_saved_bone_data() -> XrfResult {
    let original: AlifeObjectSkeleton = AlifeObjectSkeleton {
      name: String::from("test-name"),
      flags: FLAG_SKELETON_SAVED_DATA,
      source_id: 753,
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    assert!(matches!(
      original.write::<XRayByteOrder>(&mut writer),
      Err(XrfError::Parsing { message }) if message.contains("saved skeleton bones are not implemented")
    ));
    assert_eq!(writer.bytes_written(), 0);

    Ok(())
  }

  #[test]
  fn test_import_rejects_saved_bone_data() -> XrfResult {
    let mut ltx: Ltx = Ltx::new();

    AlifeObjectSkeleton {
      name: String::from("test-name"),
      flags: 0,
      source_id: 753,
    }
    .export("object", &mut ltx)?;

    ltx
      .with_section("object")
      .set("skeleton.flags", FLAG_SKELETON_SAVED_DATA.to_string());

    assert!(matches!(
      AlifeObjectSkeleton::import("object", &ltx),
      Err(XrfError::Parsing { message }) if message.contains("saved skeleton bones are not implemented")
    ));

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectSkeleton = AlifeObjectSkeleton {
      name: String::from("test-name-serde"),
      flags: 41,
      source_id: 34,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<AlifeObjectSkeleton>(&serialized)?);

    Ok(())
  }
}
