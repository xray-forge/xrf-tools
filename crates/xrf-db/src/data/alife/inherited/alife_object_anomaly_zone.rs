use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_custom_zone::AlifeObjectCustomZone;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectAnomalyZone {
  pub base: AlifeObjectCustomZone,
  pub offline_interactive_radius: f32,
  pub artefact_spawn_count: u16,
  pub artefact_position_offset: u32,
}

impl ChunkReadWrite for AlifeObjectAnomalyZone {
  /// Read anomaly zone object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      offline_interactive_radius: reader.read_f32::<T>()?,
      artefact_spawn_count: reader.read_u16::<T>()?,
      artefact_position_offset: reader.read_u32::<T>()?,
    })
  }

  /// Write anomaly zone object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_f32::<T>(self.offline_interactive_radius)?;
    writer.write_u16::<T>(self.artefact_spawn_count)?;
    writer.write_u32::<T>(self.artefact_position_offset)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectAnomalyZone {
  /// Import anomaly zone object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectCustomZone::import(section_name, ltx)?,
      offline_interactive_radius: read_ltx_field("anomaly_zone.offline_interactive_radius", section)?,
      artefact_spawn_count: read_ltx_field("anomaly_zone.artefact_spawn_count", section)?,
      artefact_position_offset: read_ltx_field("anomaly_zone.artefact_position_offset", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set(
        "anomaly_zone.offline_interactive_radius",
        self.offline_interactive_radius.to_string(),
      )
      .set(
        "anomaly_zone.artefact_spawn_count",
        self.artefact_spawn_count.to_string(),
      )
      .set(
        "anomaly_zone.artefact_position_offset",
        self.artefact_position_offset.to_string(),
      );

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
  use crate::data::alife::inherited::alife_object_anomaly_zone::AlifeObjectAnomalyZone;
  use crate::data::alife::inherited::alife_object_custom_zone::AlifeObjectCustomZone;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::generic::shape::Shape;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectAnomalyZone = AlifeObjectAnomalyZone {
      base: AlifeObjectCustomZone {
        base: AlifeObjectSpaceRestrictor {
          base: AlifeObjectAbstract {
            game_vertex_id: 241,
            distance: 55.3,
            direct_control: 4,
            level_vertex_id: 87,
            flags: 12,
            custom_data: "".to_string(),
            story_id: 6211,
            spawn_story_id: 143,
          },
          shape: vec![
            Shape::Sphere((Vector3d::new(0.5, 0.5, 0.5), 1.0)),
            Shape::Box((
              Vector3d::new(4.1, 1.1, 32.1),
              Vector3d::new(1.1, 2.2, 3.3),
              Vector3d::new(4.0, 5.0, 1.4),
              Vector3d::new(9.2, 8.3, 1.0),
            )),
          ],
          restrictor_type: 4,
        },
        max_power: 255.33,
        owner_id: 1,
        enabled_time: 3312,
        disabled_time: 521,
        start_time_shift: 250,
      },
      offline_interactive_radius: -3231.1,
      artefact_spawn_count: 3,
      artefact_position_offset: 5,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 125);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 125);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 125 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectAnomalyZone::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectAnomalyZone = AlifeObjectAnomalyZone {
      base: AlifeObjectCustomZone {
        base: AlifeObjectSpaceRestrictor {
          base: AlifeObjectAbstract {
            game_vertex_id: 4,
            distance: 250.3,
            direct_control: 40,
            level_vertex_id: 70,
            flags: 32,
            custom_data: "".to_string(),
            story_id: 523,
            spawn_story_id: 11,
          },
          shape: vec![
            Shape::Sphere((Vector3d::new(0.5, 0.5, 0.5), 1.0)),
            Shape::Box((
              Vector3d::new(40.1, 1.1, 32.1),
              Vector3d::new(1.1, 2.2, 30.3),
              Vector3d::new(4.0, 50.0, 1.4),
              Vector3d::new(90.2, 8.3, 1.0),
            )),
          ],
          restrictor_type: 4,
        },
        max_power: 29.003,
        owner_id: 10,
        enabled_time: 47,
        disabled_time: 63,
        start_time_shift: 3614,
      },
      offline_interactive_radius: -25.1,
      artefact_spawn_count: 6,
      artefact_position_offset: 5,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectAnomalyZone::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectAnomalyZone = AlifeObjectAnomalyZone {
      base: AlifeObjectCustomZone {
        base: AlifeObjectSpaceRestrictor {
          base: AlifeObjectAbstract {
            game_vertex_id: 241,
            distance: 55.3,
            direct_control: 4,
            level_vertex_id: 87,
            flags: 12,
            custom_data: "".to_string(),
            story_id: 6211,
            spawn_story_id: 143,
          },
          shape: vec![
            Shape::Sphere((Vector3d::new(0.5, 0.5, 0.5), 1.0)),
            Shape::Box((
              Vector3d::new(24.1, 10.1, 32.1),
              Vector3d::new(21.1, 20.2, 13.3),
              Vector3d::new(24.0, 50.0, 11.4),
              Vector3d::new(29.2, 80.3, 11.0),
            )),
          ],
          restrictor_type: 4,
        },
        max_power: 255.33,
        owner_id: 31,
        enabled_time: 74,
        disabled_time: 475,
        start_time_shift: 8,
      },
      offline_interactive_radius: -2.1,
      artefact_spawn_count: 6,
      artefact_position_offset: 55,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectAnomalyZone>(&serialized)?, original);

    Ok(())
  }
}
