use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_custom_zone::AlifeObjectCustomZone;
use crate::data::alife::inherited::alife_object_motion::AlifeObjectMotion;
use crate::data::generic::last_spawn_time::LastSpawnTime;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_optional_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectTorridZone {
  pub base: AlifeObjectCustomZone,
  pub motion: AlifeObjectMotion,
  pub last_spawn_time: LastSpawnTime,
}

impl ChunkReadWrite for AlifeObjectTorridZone {
  /// Read zone object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      motion: reader.read_xr::<T, _>()?,
      last_spawn_time: reader.read_xr::<T, _>()?,
    })
  }

  /// Write zone object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr::<T, _>(&self.motion)?;
    writer.write_xr::<T, _>(&self.last_spawn_time)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectTorridZone {
  /// Import torrid zone object data from ltx config section.
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
      motion: AlifeObjectMotion::import(section_name, ltx)?,
      last_spawn_time: LastSpawnTime::from_str_optional(
        read_ltx_optional_field::<String>("last_spawn_time", section)?.as_deref(),
      )?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.motion.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("last_spawn_time", self.last_spawn_time.to_ltx_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_custom_zone::AlifeObjectCustomZone;
  use crate::data::alife::inherited::alife_object_motion::AlifeObjectMotion;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::alife::inherited::alife_object_torrid_zone::AlifeObjectTorridZone;
  use crate::data::generic::last_spawn_time::LastSpawnTime;
  use crate::data::generic::time::Time;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectTorridZone = AlifeObjectTorridZone {
      base: AlifeObjectCustomZone {
        base: AlifeObjectSpaceRestrictor {
          base: AlifeObjectAbstract {
            game_vertex_id: 8469,
            distance: 85.323,
            direct_control: 203678,
            level_vertex_id: 8726,
            flags: 76,
            custom_data: String::from("custom-data"),
            story_id: 295786,
            spawn_story_id: 620,
          },
          shape: vec![],
          restrictor_type: 4,
        },
        max_power: 1.0,
        owner_id: 286,
        enabled_time: 1,
        disabled_time: 1,
        start_time_shift: 1,
      },
      motion: AlifeObjectMotion {
        motion_name: String::from("motion-name"),
      },
      last_spawn_time: LastSpawnTime::Set(Time {
        year: 12,
        month: 6,
        day: 3,
        hour: 24,
        minute: 3,
        second: 30,
        millis: 500,
      }),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 81);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 81);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 81 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectTorridZone::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }
}
