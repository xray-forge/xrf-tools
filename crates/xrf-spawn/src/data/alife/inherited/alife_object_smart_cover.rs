use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_dynamic::AlifeObjectDynamic;
use crate::data::generic::shape::Shape;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectSmartCover {
  pub base: AlifeObjectDynamic,
  pub shape: Vec<Shape>,
  pub description: String,
  pub hold_position_time: f32,
  pub enter_min_enemy_distance: f32,
  pub exit_min_enemy_distance: f32,
  pub is_combat_cover: u8,
  pub can_fire: u8,
}

impl ChunkReadWrite for AlifeObjectSmartCover {
  /// Read smart cover object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      shape: reader.read_xr_list::<T, Shape>()?,
      description: reader.read_w1251_string()?,
      hold_position_time: reader.read_f32::<T>()?,
      enter_min_enemy_distance: reader.read_f32::<T>()?,
      exit_min_enemy_distance: reader.read_f32::<T>()?,
      is_combat_cover: reader.read_u8()?,
      can_fire: reader.read_u8()?,
    })
  }

  /// Write smart cover object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr_list::<T, Shape>(&self.shape)?;
    writer.write_w1251_string(&self.description)?;
    writer.write_f32::<T>(self.hold_position_time)?;
    writer.write_f32::<T>(self.enter_min_enemy_distance)?;
    writer.write_f32::<T>(self.exit_min_enemy_distance)?;
    writer.write_u8(self.is_combat_cover)?;
    writer.write_u8(self.can_fire)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectSmartCover {
  /// Import smart cover object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamic::import(section_name, ltx)?,
      shape: Shape::import_list(section)?,
      description: read_ltx_field("description", section)?,
      hold_position_time: read_ltx_field("hold_position_time", section)?,
      enter_min_enemy_distance: read_ltx_field("enter_min_enemy_distance", section)?,
      exit_min_enemy_distance: read_ltx_field("exit_min_enemy_distance", section)?,
      is_combat_cover: read_ltx_field("is_combat_cover", section)?,
      can_fire: read_ltx_field("can_fire", section)?,
    })
  }
  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("description", &self.description)
      .set("hold_position_time", self.hold_position_time.to_string())
      .set("enter_min_enemy_distance", self.enter_min_enemy_distance.to_string())
      .set("exit_min_enemy_distance", self.exit_min_enemy_distance.to_string())
      .set("exit_min_enemy_distance", self.exit_min_enemy_distance.to_string())
      .set("is_combat_cover", self.is_combat_cover.to_string())
      .set("can_fire", self.can_fire.to_string());

    Shape::export_list(&self.shape, section_name, ltx);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic::AlifeObjectDynamic;
  use crate::data::alife::inherited::alife_object_smart_cover::AlifeObjectSmartCover;
  use crate::data::generic::shape::Shape;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectSmartCover = AlifeObjectSmartCover {
      base: AlifeObjectDynamic {
        base: AlifeObjectAbstract {
          game_vertex_id: 1001,
          distance: 65.25,
          direct_control: 412421,
          level_vertex_id: 66231,
          flags: 33,
          custom_data: String::from("custom_data"),
          story_id: 400,
          spawn_story_id: 32,
        },
      },
      shape: vec![
        Shape::Sphere((Vector3d::new(0.5, 0.3, -0.125), 2.5)),
        Shape::Box((
          Vector3d::new(1.1, 1.1, 3.1),
          Vector3d::new(1.4, 2.2, 3.3),
          Vector3d::new(4.0, 3.0, 5.4),
          Vector3d::new(9.2, 8.3, 2.0),
        )),
      ],
      description: String::from("test-description"),
      hold_position_time: 4.532,
      enter_min_enemy_distance: 32.4,
      exit_min_enemy_distance: 25.3,
      is_combat_cover: 0,
      can_fire: 1,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 136);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 136);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 136 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectSmartCover::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }
}
