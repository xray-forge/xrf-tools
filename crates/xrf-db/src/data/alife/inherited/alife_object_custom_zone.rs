use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectCustomZone {
  pub base: AlifeObjectSpaceRestrictor,
  pub max_power: f32,
  pub owner_id: u32,
  pub enabled_time: u32,
  pub disabled_time: u32,
  pub start_time_shift: u32,
}

impl ChunkReadWrite for AlifeObjectCustomZone {
  /// Read ALife custom zone object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      max_power: reader.read_f32::<T>()?,
      owner_id: reader.read_u32::<T>()?,
      enabled_time: reader.read_u32::<T>()?,
      disabled_time: reader.read_u32::<T>()?,
      start_time_shift: reader.read_u32::<T>()?,
    })
  }

  /// Write custom zone object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_f32::<T>(self.max_power)?;
    writer.write_u32::<T>(self.owner_id)?;
    writer.write_u32::<T>(self.enabled_time)?;
    writer.write_u32::<T>(self.disabled_time)?;
    writer.write_u32::<T>(self.start_time_shift)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectCustomZone {
  /// Import ALife custom zone object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectSpaceRestrictor::import(section_name, ltx)?,
      max_power: read_ltx_field("custom_zone.max_power", section)?,
      owner_id: read_ltx_field("custom_zone.owner_id", section)?,
      enabled_time: read_ltx_field("custom_zone.enabled_time", section)?,
      disabled_time: read_ltx_field("custom_zone.disabled_time", section)?,
      start_time_shift: read_ltx_field("custom_zone.start_time_shift", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("custom_zone.max_power", self.max_power.to_string())
      .set("custom_zone.owner_id", self.owner_id.to_string())
      .set("custom_zone.enabled_time", self.enabled_time.to_string())
      .set("custom_zone.disabled_time", self.disabled_time.to_string())
      .set("custom_zone.start_time_shift", self.start_time_shift.to_string());

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
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_custom_zone::AlifeObjectCustomZone;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::generic::shape::Shape;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectCustomZone = AlifeObjectCustomZone {
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          game_vertex_id: 42343,
          distance: 255.4,
          direct_control: 3,
          level_vertex_id: 1003,
          flags: 32,
          custom_data: String::from("custom-data"),
          story_id: 441,
          spawn_story_id: 254,
        },
        shape: vec![
          Shape::Sphere((Vector3d::new(2.5, 3.5, 1.5), 1.0)),
          Shape::Box((
            Vector3d::new(1.1, 1.1, 3.1),
            Vector3d::new(1.1, 2.2, 3.3),
            Vector3d::new(4.0, 5.0, 6.4),
            Vector3d::new(9.2, 8.3, 7.0),
          )),
        ],
        restrictor_type: 3,
      },
      max_power: 2.0,
      owner_id: 553,
      enabled_time: 100,
      disabled_time: 220,
      start_time_shift: 300,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 126);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 126);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 126 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectCustomZone::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectCustomZone = AlifeObjectCustomZone {
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          game_vertex_id: 14,
          distance: 25.4,
          direct_control: 5,
          level_vertex_id: 362,
          flags: 16,
          custom_data: String::from("custom-data"),
          story_id: 45,
          spawn_story_id: 52,
        },
        shape: vec![
          Shape::Sphere((Vector3d::new(2.5, 3.5, 1.5), 1.0)),
          Shape::Box((
            Vector3d::new(11.1, 21.1, 33.1),
            Vector3d::new(11.1, 22.2, 33.3),
            Vector3d::new(14.0, 25.0, 36.4),
            Vector3d::new(19.2, 28.3, 37.0),
          )),
        ],
        restrictor_type: 3,
      },
      max_power: 2.0,
      owner_id: 53,
      enabled_time: 25,
      disabled_time: 677,
      start_time_shift: 63,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectCustomZone::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectCustomZone = AlifeObjectCustomZone {
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          game_vertex_id: 14,
          distance: 25.4,
          direct_control: 5,
          level_vertex_id: 55,
          flags: 16,
          custom_data: String::from("custom-data"),
          story_id: 45,
          spawn_story_id: 52,
        },
        shape: vec![
          Shape::Sphere((Vector3d::new(22.5, 13.5, 31.5), 15.0)),
          Shape::Box((
            Vector3d::new(25.1, 21.1, 33.1),
            Vector3d::new(25.1, 22.2, 33.3),
            Vector3d::new(25.0, 25.0, 36.4),
            Vector3d::new(25.2, 28.3, 37.0),
          )),
        ],
        restrictor_type: 3,
      },
      max_power: 2.0,
      owner_id: 6,
      enabled_time: 12,
      disabled_time: 251,
      start_time_shift: 45,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectCustomZone>(&serialized)?, original);

    Ok(())
  }
}
