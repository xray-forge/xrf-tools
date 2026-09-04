use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::assert_equal;

use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
use crate::data::generic::vector_3d::Vector3d;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeLevelChanger {
  pub base: AlifeObjectSpaceRestrictor,
  pub dest_game_vertex_id: u16,
  pub dest_level_vertex_id: u32,
  pub dest_position: Vector3d<f32>,
  pub dest_direction: Vector3d<f32>,
  pub angle_y: f32,
  pub dest_level_name: String,
  pub dest_graph_point: String,
  pub silent_mode: u8,
  pub enabled: u8,
  pub hint: String,
  pub save_marker: u16,
}

impl ChunkReadWrite for AlifeLevelChanger {
  /// Read ALife level changer object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let object: Self = Self {
      base: reader.read_xr::<T, _>()?,
      dest_game_vertex_id: reader.read_u16::<T>()?,
      dest_level_vertex_id: reader.read_u32::<T>()?,
      dest_position: reader.read_xr::<T, _>()?,
      dest_direction: reader.read_xr::<T, _>()?,
      angle_y: reader.read_f32::<T>()?,
      dest_level_name: reader.read_w1251_string()?,
      dest_graph_point: reader.read_w1251_string()?,
      silent_mode: reader.read_u8()?,
      enabled: reader.read_u8()?,
      hint: reader.read_w1251_string()?,
      save_marker: reader.read_u16::<T>()?,
    };

    assert_equal(
      object.save_marker,
      26,
      "Unexpected script data provided for level changer",
    )?;

    Ok(object)
  }

  /// Write object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;

    writer.write_u16::<XRayByteOrder>(self.dest_game_vertex_id)?;
    writer.write_u32::<XRayByteOrder>(self.dest_level_vertex_id)?;

    writer.write_xr::<XRayByteOrder, _>(&self.dest_position)?;
    writer.write_xr::<XRayByteOrder, _>(&self.dest_direction)?;

    writer.write_f32::<XRayByteOrder>(self.angle_y)?;
    writer.write_w1251_string(&self.dest_level_name)?;
    writer.write_w1251_string(&self.dest_graph_point)?;
    writer.write_u8(self.silent_mode)?;

    writer.write_u8(self.enabled)?;
    writer.write_w1251_string(&self.hint)?;
    writer.write_u16::<XRayByteOrder>(self.save_marker)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeLevelChanger {
  /// Import ALife level changer object data from ltx config section.
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
      dest_game_vertex_id: read_ltx_field("level_changer.dest_game_vertex_id", section)?,
      dest_level_vertex_id: read_ltx_field("level_changer.dest_level_vertex_id", section)?,
      dest_position: read_ltx_field("level_changer.dest_position", section)?,
      dest_direction: read_ltx_field("level_changer.dest_direction", section)?,
      angle_y: read_ltx_field("level_changer.angle_y", section)?,
      dest_level_name: read_ltx_field("level_changer.dest_level_name", section)?,
      dest_graph_point: read_ltx_field("level_changer.dest_graph_point", section)?,
      silent_mode: read_ltx_field("level_changer.silent_mode", section)?,
      enabled: read_ltx_field("level_changer.enabled", section)?,
      hint: read_ltx_field("level_changer.hint", section)?,
      save_marker: read_ltx_field("level_changer.save_marker", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set(
        "level_changer.dest_game_vertex_id",
        self.dest_game_vertex_id.to_string(),
      )
      .set(
        "level_changer.dest_level_vertex_id",
        self.dest_level_vertex_id.to_string(),
      )
      .set("level_changer.dest_position", self.dest_position.to_string())
      .set("level_changer.dest_direction", self.dest_direction.to_string())
      .set("level_changer.angle_y", self.angle_y.to_string())
      .set("level_changer.dest_level_name", &self.dest_level_name)
      .set("level_changer.dest_graph_point", &self.dest_graph_point)
      .set("level_changer.silent_mode", self.silent_mode.to_string())
      .set("level_changer.enabled", self.enabled.to_string())
      .set("level_changer.hint", &self.hint)
      .set("level_changer.save_marker", self.save_marker.to_string());

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

  use crate::data::alife::inherited::alife_level_changer::AlifeLevelChanger;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::generic::shape::Shape;
  use crate::data::generic::vector_3d::Vector3d;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeLevelChanger = AlifeLevelChanger {
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          game_vertex_id: 12451,
          distance: 253.0,
          direct_control: 12,
          level_vertex_id: 331,
          flags: 33,
          custom_data: String::from("custom-data"),
          story_id: 4553,
          spawn_story_id: 213,
        },
        shape: vec![
          Shape::Sphere((Vector3d::new(4.5, 0.5, 11.5), 1.0)),
          Shape::Box((
            Vector3d::new(1.5, 2.5, 73.1),
            Vector3d::new(5.1, 3.2, 2.3),
            Vector3d::new(1.0, 3.0, 6.4),
            Vector3d::new(9.2, 3.3, 3.0),
          )),
        ],
        restrictor_type: 3,
      },
      dest_game_vertex_id: 312,
      dest_level_vertex_id: 3312,
      dest_position: Vector3d::new(4.0, 3.0, 2.0),
      dest_direction: Vector3d::new(1.0, 2.0, 3.0),
      angle_y: 35.0,
      dest_level_name: String::from("dest-level"),
      dest_graph_point: String::from("dest-graph-point"),
      silent_mode: 1,
      enabled: 1,
      hint: String::from("hint"),
      save_marker: 26,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 177);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 177);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 177 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;
    let read_object: AlifeLevelChanger = AlifeLevelChanger::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read_object, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeLevelChanger = AlifeLevelChanger {
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          game_vertex_id: 7,
          distance: 1.0,
          direct_control: 25,
          level_vertex_id: 14,
          flags: 64,
          custom_data: String::from("custom-data"),
          story_id: 47,
          spawn_story_id: 213,
        },
        shape: vec![
          Shape::Sphere((Vector3d::new(4.5, 0.5, 11.5), 1.0)),
          Shape::Box((
            Vector3d::new(21.5, 32.5, 73.1),
            Vector3d::new(25.1, 33.2, 42.3),
            Vector3d::new(21.0, 33.0, 46.4),
            Vector3d::new(29.2, 33.3, 43.0),
          )),
        ],
        restrictor_type: 3,
      },
      dest_game_vertex_id: 312,
      dest_level_vertex_id: 3312,
      dest_position: Vector3d::new(4.0, 3.0, 5.0),
      dest_direction: Vector3d::new(5.0, 2.0, 3.0),
      angle_y: 35.0,
      dest_level_name: String::from("dest-level"),
      dest_graph_point: String::from("dest-graph-point"),
      silent_mode: 1,
      enabled: 0,
      hint: String::from("hint"),
      save_marker: 27,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeLevelChanger::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeLevelChanger = AlifeLevelChanger {
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          game_vertex_id: 5,
          distance: 10.0,
          direct_control: 15,
          level_vertex_id: 35,
          flags: 12,
          custom_data: String::from("custom-data"),
          story_id: 45,
          spawn_story_id: 1,
        },
        shape: vec![
          Shape::Sphere((Vector3d::new(4.5, 0.5, 11.5), 1.0)),
          Shape::Box((
            Vector3d::new(10.5, 20.5, 73.1),
            Vector3d::new(5.1, 30.2, 2.3),
            Vector3d::new(1.0, 3.0, 60.4),
            Vector3d::new(90.2, 30.3, 3.0),
          )),
        ],
        restrictor_type: 3,
      },
      dest_game_vertex_id: 312,
      dest_level_vertex_id: 3312,
      dest_position: Vector3d::new(40.0, 30.0, 20.0),
      dest_direction: Vector3d::new(10.0, 20.0, 30.0),
      angle_y: 350.0,
      dest_level_name: String::from("dest-level"),
      dest_graph_point: String::from("dest-graph-point"),
      silent_mode: 0,
      enabled: 1,
      hint: String::from("hint"),
      save_marker: 26,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeLevelChanger>(&serialized)?, original);

    Ok(())
  }
}
