use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};
use xrf_utils::assert_equal;

use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeActor {
  pub base: AlifeObjectActor,
  pub start_position_filled: u8,
  pub save_marker: u16,
}

impl ChunkReadWrite for AlifeActor {
  /// Read actor data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let object: Self = Self {
      base: reader.read_xr::<T, _>()?,
      start_position_filled: reader.read_u8()?,
      save_marker: reader.read_u16::<T>()?,
    };

    assert_equal(object.save_marker, 1, "Unexpected save data for actor object provided")?;

    Ok(object)
  }

  /// Write object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_u8(self.start_position_filled)?;
    writer.write_u16::<T>(self.save_marker)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeActor {
  /// Import actor data from ltx file section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectActor::import(section_name, ltx)?,
      start_position_filled: read_ltx_field("actor.start_position_filled", section)?,
      save_marker: read_ltx_field("actor.save_marker", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("actor.start_position_filled", self.start_position_filled.to_string())
      .set("actor.save_marker", self.save_marker.to_string());

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
  use xrf_ltx::LtxImportExport;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_actor::AlifeActor;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;
  use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeActor = AlifeActor {
      base: AlifeObjectActor {
        base: AlifeObjectCreature {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 621,
              distance: 55.25,
              direct_control: 15,
              level_vertex_id: 52235,
              flags: 72,
              custom_data: String::from("custom-data"),
              story_id: 15,
              spawn_story_id: 334,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 13,
          },
          team: 2,
          squad: 3,
          group: 4,
          health: 1.0,
          dynamic_out_restrictions: vec![1, 2, 3, 4],
          dynamic_in_restrictions: vec![5, 6, 7, 8],
          killer_id: 0,
          game_death_time: 0,
        },
        trader: AlifeObjectTraderAbstract {
          money: 5000,
          specific_character: String::from("specific-character-0"),
          trader_flags: 23,
          character_profile: String::from("character-profile-0"),
          community_index: 1,
          rank: 2,
          reputation: 3,
          character_name: String::from("character-name-0"),
          dead_body_can_take: 1,
          dead_body_closed: 1,
        },
        skeleton: AlifeObjectSkeleton {
          name: String::from("skeleton-name-0"),
          flags: 98,
          source_id: 12,
        },
        holder_id: 0,
      },
      start_position_filled: 1,
      save_marker: 1,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 196);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 196);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 196 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeActor::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeActor = AlifeActor {
      base: AlifeObjectActor {
        base: AlifeObjectCreature {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 1,
              distance: 40.25,
              direct_control: 16,
              level_vertex_id: 523,
              flags: 14,
              custom_data: String::from("custom-data"),
              story_id: 16,
              spawn_story_id: 334,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 13,
          },
          team: 2,
          squad: 3,
          group: 5,
          health: 1.0,
          dynamic_out_restrictions: vec![1, 2, 3, 4],
          dynamic_in_restrictions: vec![5, 6, 7, 8],
          killer_id: 0,
          game_death_time: 0,
        },
        trader: AlifeObjectTraderAbstract {
          money: 6000,
          specific_character: String::from("specific-character-0"),
          trader_flags: 24,
          character_profile: String::from("character-profile-0"),
          community_index: 2,
          rank: 4,
          reputation: 5,
          character_name: String::from("character-name-0"),
          dead_body_can_take: 1,
          dead_body_closed: 1,
        },
        skeleton: AlifeObjectSkeleton {
          name: String::from("skeleton-name-0"),
          flags: 25,
          source_id: 12,
        },
        holder_id: 0,
      },
      start_position_filled: 1,
      save_marker: 1,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeActor::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeActor = AlifeActor {
      base: AlifeObjectActor {
        base: AlifeObjectCreature {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 5,
              distance: 10.25,
              direct_control: 16,
              level_vertex_id: 523,
              flags: 12,
              custom_data: String::from("custom-data"),
              story_id: 16,
              spawn_story_id: 26,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 17,
          },
          team: 2,
          squad: 3,
          group: 5,
          health: 1.0,
          dynamic_out_restrictions: vec![1, 2, 3, 4],
          dynamic_in_restrictions: vec![5, 6, 7, 8],
          killer_id: 0,
          game_death_time: 0,
        },
        trader: AlifeObjectTraderAbstract {
          money: 3000,
          specific_character: String::from("specific-character-0"),
          trader_flags: 24,
          character_profile: String::from("character-profile-0"),
          community_index: 5,
          rank: 4,
          reputation: 5,
          character_name: String::from("character-name-0"),
          dead_body_can_take: 0,
          dead_body_closed: 0,
        },
        skeleton: AlifeObjectSkeleton {
          name: String::from("skeleton-name-0"),
          flags: 27,
          source_id: 12,
        },
        holder_id: 0,
      },
      start_position_filled: 1,
      save_marker: 1,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeActor>(&serialized)?, original);

    Ok(())
  }
}
