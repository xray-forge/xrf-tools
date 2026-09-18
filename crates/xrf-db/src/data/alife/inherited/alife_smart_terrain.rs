use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::assert_equal;

use crate::data::alife::inherited::alife_smart_zone::AlifeSmartZone;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeSmartTerrain {
  pub base: AlifeSmartZone,
  pub arriving_objects_count: u8,
  pub object_job_descriptors_count: u8,
  pub dead_objects_infos_count: u8,
  pub smart_terrain_actor_control: u8,
  pub respawn_point: u8,
  pub staying_objects_count: u8,
  pub save_marker: u16,
}

impl ChunkReadWrite for AlifeSmartTerrain {
  /// Read ALife smart terrain data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let base: AlifeSmartZone = AlifeSmartZone::read::<T, _>(reader)?;

    let arriving_objects_count: u8 = reader.read_u8()?;

    assert_equal(
      arriving_objects_count,
      0,
      "Unexpected arriving objects in smart terrain",
    )?;

    let object_job_descriptors_count: u8 = reader.read_u8()?;

    assert_equal(
      object_job_descriptors_count,
      0,
      "Unexpected job objects in smart terrain",
    )?;

    let dead_objects_infos_count: u8 = reader.read_u8()?;

    assert_equal(dead_objects_infos_count, 0, "Unexpected dead objects in smart terrain")?;

    let smart_terrain_actor_control: u8 = reader.read_u8()?;

    assert_equal(smart_terrain_actor_control, 0, "Unexpected smart terrain actor control")?;

    let respawn_point: u8 = reader.read_u8()?;

    if respawn_point != 0 {
      return Err(XrfError::new_parsing_error(
        "Unexpected respawn point handler in smart terrain parser",
      ));
    }

    let staying_objects_count: u8 = reader.read_u8()?;

    assert_equal(staying_objects_count, 0, "Unexpected smart terrain staying objects")?;

    let save_marker: u16 = reader.read_u16::<T>()?;

    assert_equal(save_marker, 6, "Unexpected data provided with smart terrain save")?;

    Ok(Self {
      base,
      arriving_objects_count,
      object_job_descriptors_count,
      dead_objects_infos_count,
      smart_terrain_actor_control,
      respawn_point,
      staying_objects_count,
      save_marker,
    })
  }

  /// Write smart terrain data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_u8(self.arriving_objects_count)?;
    writer.write_u8(self.object_job_descriptors_count)?;
    writer.write_u8(self.dead_objects_infos_count)?;
    writer.write_u8(self.smart_terrain_actor_control)?;
    writer.write_u8(self.respawn_point)?;
    writer.write_u8(self.staying_objects_count)?;
    writer.write_u16::<T>(self.save_marker)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeSmartTerrain {
  /// Import ALife smart terrain data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeSmartZone::import(section_name, ltx)?,
      arriving_objects_count: read_ltx_field("arriving_objects_count", section)?,
      object_job_descriptors_count: read_ltx_field("object_job_descriptors_count", section)?,
      dead_objects_infos_count: read_ltx_field("dead_objects_infos_count", section)?,
      smart_terrain_actor_control: read_ltx_field("smart_terrain_actor_control", section)?,
      respawn_point: read_ltx_field("respawn_point", section)?,
      staying_objects_count: read_ltx_field("staying_objects_count", section)?,
      save_marker: read_ltx_field("save_marker", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("arriving_objects_count", self.arriving_objects_count.to_string())
      .set(
        "object_job_descriptors_count",
        self.object_job_descriptors_count.to_string(),
      )
      .set("dead_objects_infos_count", self.dead_objects_infos_count.to_string())
      .set(
        "smart_terrain_actor_control",
        self.smart_terrain_actor_control.to_string(),
      )
      .set("respawn_point", self.respawn_point.to_string())
      .set("staying_objects_count", self.staying_objects_count.to_string())
      .set("save_marker", self.save_marker.to_string());

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
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::alife::inherited::alife_smart_terrain::AlifeSmartTerrain;
  use crate::data::alife::inherited::alife_smart_zone::AlifeSmartZone;
  use crate::data::generic::shape::Shape;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeSmartTerrain = AlifeSmartTerrain {
      base: AlifeSmartZone {
        base: AlifeObjectSpaceRestrictor {
          base: AlifeObjectAbstract {
            game_vertex_id: 1002,
            distance: 65.25,
            direct_control: 31231,
            level_vertex_id: 3213,
            flags: 34,
            custom_data: String::from("custom_data"),
            story_id: 400,
            spawn_story_id: 25,
          },
          shape: vec![
            Shape::Sphere((Vector3d::new(3.5, -2.5, 11.5), 1.0)),
            Shape::Box((
              Vector3d::new(1.5, 1.1, 73.1),
              Vector3d::new(5.1, 2.2, 3.3),
              Vector3d::new(4.0, 6.0, 2.4),
              Vector3d::new(9.2, 4.3, 3.0),
            )),
          ],
          restrictor_type: 2,
        },
      },
      arriving_objects_count: 0,
      object_job_descriptors_count: 0,
      dead_objects_infos_count: 0,
      smart_terrain_actor_control: 0,
      respawn_point: 0,
      staying_objects_count: 0,
      save_marker: 6,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 114);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 114);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 114 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeSmartTerrain::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }
}
