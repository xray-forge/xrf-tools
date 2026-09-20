use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section, read_ltx_field};

use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::generic::shape::Shape;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectSpaceRestrictor {
  pub base: AlifeObjectAbstract,
  pub shape: Vec<Shape>,
  pub restrictor_type: u8,
}

impl ChunkReadWrite for AlifeObjectSpaceRestrictor {
  /// Read generic space restrictor data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      shape: reader.read_xr_list::<T, Shape>()?,
      restrictor_type: reader.read_u8()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr_list::<T, Shape>(&self.shape)?;
    writer.write_u8(self.restrictor_type)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectSpaceRestrictor {
  /// Import generic space restrictor data from the chunk.
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
      shape: Shape::import_list(section)?,
      restrictor_type: read_ltx_field("space_restrictor.restrictor_type", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    Shape::export_list(&self.shape, section_name, ltx);

    ltx
      .with_section(section_name)
      .set("space_restrictor.restrictor_type", self.restrictor_type.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_file, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::generic::shape::Shape;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectSpaceRestrictor = AlifeObjectSpaceRestrictor {
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
      shape: vec![
        Shape::Sphere((Vector3d::new(0.5, 0.5, 0.5), 1.0)),
        Shape::Box((
          Vector3d::new(1.1, 1.1, 3.1),
          Vector3d::new(1.1, 2.2, 3.3),
          Vector3d::new(4.0, 5.0, 6.4),
          Vector3d::new(9.2, 8.3, 7.0),
        )),
      ],
      restrictor_type: 2,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 106);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 106);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 106 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(
      AlifeObjectSpaceRestrictor::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let mut file: File = overwrite_file(config_path)?;
    let mut ltx: Ltx = Ltx::new();

    let first: AlifeObjectSpaceRestrictor = AlifeObjectSpaceRestrictor {
      base: AlifeObjectAbstract {
        game_vertex_id: 2593,
        distance: 34.7,
        direct_control: 235,
        level_vertex_id: 245423,
        flags: 32,
        custom_data: String::from("test-custom-data"),
        story_id: 253423,
        spawn_story_id: 457,
      },
      shape: vec![],
      restrictor_type: 3,
    };

    let second: AlifeObjectSpaceRestrictor = AlifeObjectSpaceRestrictor {
      base: AlifeObjectAbstract {
        game_vertex_id: 45724,
        distance: 43.0,
        direct_control: 236623,
        level_vertex_id: 2364,
        flags: 75,
        custom_data: String::new(),
        story_id: 253,
        spawn_story_id: 7546,
      },
      shape: vec![
        Shape::Sphere((Vector3d::new(54.5, 0.5, 11.5), 1.0)),
        Shape::Box((
          Vector3d::new(3.5, 2.5, 73.1),
          Vector3d::new(55.1, 1.2, 2.3),
          Vector3d::new(51.0, 7.0, 3.4),
          Vector3d::new(59.2, 3.3, 4.1),
        )),
      ],
      restrictor_type: 4,
    };

    first.export("first", &mut ltx)?;
    second.export("second", &mut ltx)?;

    ltx.write_to(&mut file)?;

    let source: Ltx = Ltx::read_from_path(config_path)?;

    assert_eq!(AlifeObjectSpaceRestrictor::import("first", &source)?, first);
    assert_eq!(AlifeObjectSpaceRestrictor::import("second", &source)?, second);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectSpaceRestrictor = AlifeObjectSpaceRestrictor {
      base: AlifeObjectAbstract {
        game_vertex_id: 4,
        distance: 2.7,
        direct_control: 10,
        level_vertex_id: 25,
        flags: 64,
        custom_data: String::from("test-custom-data"),
        story_id: 256,
        spawn_story_id: 3,
      },
      shape: vec![
        Shape::Sphere((Vector3d::new(54.5, 0.5, 11.5), 1.0)),
        Shape::Box((
          Vector3d::new(23.5, 2.5, 73.1),
          Vector3d::new(25.1, 1.2, 2.3),
          Vector3d::new(21.0, 7.0, 3.4),
          Vector3d::new(26.2, 3.3, 4.1),
        )),
      ],
      restrictor_type: 6,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(
      serde_json::from_str::<AlifeObjectSpaceRestrictor>(&serialized)?,
      original
    );

    Ok(())
  }
}
