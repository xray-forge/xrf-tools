use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::to_format_size;

use crate::data::alife::inherited::alife_object_smart_cover::AlifeObjectSmartCover;
use crate::data::alife::inherited::alife_smart_cover_loophole::AlifeSmartCoverLoophole;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

/// Represents script extension of base server smart cover class.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeSmartCover {
  pub base: AlifeObjectSmartCover,
  pub last_description: String,
  pub loopholes: Vec<AlifeSmartCoverLoophole>,
}

impl ChunkReadWrite for AlifeSmartCover {
  /// Read smart cover data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let base: AlifeObjectSmartCover = reader.read_xr::<T, _>()?;

    let last_description: String = reader.read_w1251_string()?;
    let count: u8 = reader.read_u8()?;
    let mut loopholes: Vec<AlifeSmartCoverLoophole> = reader.new_bounded_vec(
      count.into(),
      AlifeSmartCoverLoophole::MIN_SERIALIZED_SIZE,
      "smart cover loopholes",
    )?;

    for _ in 0..count {
      let name: String = reader.read_w1251_string()?;
      let enabled: u8 = reader.read_u8()?;

      loopholes.push(AlifeSmartCoverLoophole { name, enabled })
    }

    Ok(Self {
      base,
      last_description,
      loopholes,
    })
  }

  /// Write smart cover data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_w1251_string(&self.last_description)?;
    writer.write_u8(to_format_size(self.loopholes.len(), "smart cover loopholes")?)?;

    for loophole in &self.loopholes {
      writer.write_w1251_string(&loophole.name)?;
      writer.write_u8(loophole.enabled)?;
    }

    Ok(())
  }
}

impl LtxImportExport for AlifeSmartCover {
  /// Import smart cover data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectSmartCover::import(section_name, ltx)?,
      last_description: read_ltx_field("last_description", section)?,
      loopholes: AlifeSmartCoverLoophole::string_to_list(&read_ltx_field::<String>("loopholes", section)?)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("last_description", &self.last_description)
      .set("loopholes", self.loopholes.len().to_string())
      .set("loopholes", AlifeSmartCoverLoophole::list_to_string(&self.loopholes));

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic::AlifeObjectDynamic;
  use crate::data::alife::inherited::alife_object_smart_cover::AlifeObjectSmartCover;
  use crate::data::alife::inherited::alife_smart_cover::AlifeSmartCover;
  use crate::data::alife::inherited::alife_smart_cover_loophole::AlifeSmartCoverLoophole;
  use crate::data::generic::shape::Shape;
  use crate::data::generic::vector_3d::Vector3d;

  #[test]
  fn rejects_a_loophole_list_past_its_count_field() {
    let original: AlifeSmartCover = AlifeSmartCover {
      base: AlifeObjectSmartCover {
        base: AlifeObjectDynamic {
          base: AlifeObjectAbstract {
            game_vertex_id: 6734,
            distance: 38.287,
            direct_control: 234760,
            level_vertex_id: 29836,
            flags: 68,
            custom_data: String::from("custom-data"),
            story_id: 8723,
            spawn_story_id: 160278,
          },
        },
        shape: vec![Shape::Sphere((Vector3d::new(2.5, 1.3, -4.125), 5.5))],
        description: String::from("description"),
        hold_position_time: 34.0,
        enter_min_enemy_distance: 23.0,
        exit_min_enemy_distance: 36.0,
        is_combat_cover: 1,
        can_fire: 1,
      },
      last_description: String::from("last-description"),
      loopholes: vec![
        AlifeSmartCoverLoophole {
          name: String::from("loophole"),
          enabled: 1,
        };
        256
      ],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      original
        .write::<XRayByteOrder>(&mut writer)
        .expect_err("expect the loophole count to exceed its format field")
        .to_string(),
      "Invalid error: smart cover loopholes exceeds the u8 format limit"
    );
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectSmartCover = AlifeObjectSmartCover {
      base: AlifeObjectDynamic {
        base: AlifeObjectAbstract {
          game_vertex_id: 6734,
          distance: 38.287,
          direct_control: 234760,
          level_vertex_id: 29836,
          flags: 68,
          custom_data: String::from("custom-data"),
          story_id: 8723,
          spawn_story_id: 160278,
        },
      },
      shape: vec![
        Shape::Sphere((Vector3d::new(2.5, 1.3, -4.125), 5.5)),
        Shape::Box((
          Vector3d::new(1.1, 1.1, 6.1),
          Vector3d::new(1.4, 2.2, 6.3),
          Vector3d::new(4.0, 3.0, 6.4),
          Vector3d::new(9.2, 8.3, 6.0),
        )),
      ],
      description: String::from("description"),
      hold_position_time: 34.0,
      enter_min_enemy_distance: 23.0,
      exit_min_enemy_distance: 36.0,
      is_combat_cover: 1,
      can_fire: 1,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 131);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 131);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 131 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectSmartCover::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn rejects_loophole_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let original: AlifeSmartCover = AlifeSmartCover {
      base: AlifeObjectSmartCover {
        base: AlifeObjectDynamic {
          base: AlifeObjectAbstract {
            game_vertex_id: 12,
            distance: 1.5,
            direct_control: 24,
            level_vertex_id: 36,
            flags: 0,
            custom_data: String::from("custom-data"),
            story_id: 48,
            spawn_story_id: 60,
          },
        },
        shape: vec![Shape::Sphere((Vector3d::new(1.0, 2.0, 3.0), 4.0))],
        description: String::from("description"),
        hold_position_time: 1.0,
        enter_min_enemy_distance: 2.0,
        exit_min_enemy_distance: 3.0,
        is_combat_cover: 1,
        can_fire: 1,
      },
      last_description: String::from("last-description"),
      loopholes: vec![],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    // The loophole count is the last byte written, so raising it declares loopholes the payload cannot hold.
    let mut bytes: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?;

    *bytes.last_mut().expect("expect a trailing loophole count") = 255;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?.read_child_by_index(0)?;

    let error: String = AlifeSmartCover::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared loophole count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("smart cover loopholes declares 255 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
