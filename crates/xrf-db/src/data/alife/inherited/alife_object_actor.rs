use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectActor {
  pub base: AlifeObjectCreature,
  pub trader: AlifeObjectTraderAbstract,
  pub skeleton: AlifeObjectSkeleton,
  pub holder_id: u16,
}

impl ChunkReadWrite for AlifeObjectActor {
  /// Read actor data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      trader: reader.read_xr::<T, _>()?,
      skeleton: reader.read_xr::<T, _>()?,
      holder_id: reader.read_u16::<T>()?,
    })
  }

  /// Write object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr::<T, _>(&self.trader)?;
    writer.write_xr::<T, _>(&self.skeleton)?;
    writer.write_u16::<T>(self.holder_id)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectActor {
  /// Import actor data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectCreature::import(section_name, ltx)?,
      trader: AlifeObjectTraderAbstract::import(section_name, ltx)?,
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
      holder_id: read_ltx_field("actor.holder_id", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.trader.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("actor.holder_id", self.holder_id.to_string());

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
  use xrf_ltx::Ltx;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_file, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;
  use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectActor = AlifeObjectActor {
      base: AlifeObjectCreature {
        base: AlifeObjectDynamicVisual {
          base: AlifeObjectAbstract {
            game_vertex_id: 620,
            distance: 42.25,
            direct_control: 14,
            level_vertex_id: 52234,
            flags: 71,
            custom_data: String::from("custom-data"),
            story_id: 14,
            spawn_story_id: 336,
          },
          visual_name: String::from("visual-name"),
          visual_flags: 12,
        },
        team: 1,
        squad: 2,
        group: 3,
        health: 1.0,
        dynamic_out_restrictions: vec![1, 2, 3, 4],
        dynamic_in_restrictions: vec![5, 6, 7, 8],
        killer_id: 0,
        game_death_time: 0,
      },
      trader: AlifeObjectTraderAbstract {
        money: 5000,
        specific_character: String::from("specific-character"),
        trader_flags: 23,
        character_profile: String::from("character-profile"),
        community_index: 1,
        rank: 2,
        reputation: 3,
        character_name: String::from("character-name"),
        dead_body_can_take: 1,
        dead_body_closed: 1,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 98,
        source_id: 12,
      },
      holder_id: 0,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 185);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 185);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 185 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectActor::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let mut file: File = overwrite_file(config_path)?;
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectActor = AlifeObjectActor {
      base: AlifeObjectCreature {
        base: AlifeObjectDynamicVisual {
          base: AlifeObjectAbstract {
            game_vertex_id: 6,
            distance: 25.25,
            direct_control: 3,
            level_vertex_id: 5286,
            flags: 45,
            custom_data: String::from("custom-data"),
            story_id: 10,
            spawn_story_id: 33,
          },
          visual_name: String::from("visual-name"),
          visual_flags: 14,
        },
        team: 1,
        squad: 3,
        group: 2,
        health: 1.0,
        dynamic_out_restrictions: vec![1, 2, 3, 4],
        dynamic_in_restrictions: vec![5, 6, 7, 8],
        killer_id: 0,
        game_death_time: 0,
      },
      trader: AlifeObjectTraderAbstract {
        money: 6000,
        specific_character: String::from("specific-character"),
        trader_flags: 25,
        character_profile: String::from("character-profile"),
        community_index: 1,
        rank: 2,
        reputation: 4,
        character_name: String::from("character-name"),
        dead_body_can_take: 1,
        dead_body_closed: 0,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 41,
        source_id: 10,
      },
      holder_id: 4,
    };

    original.export("first", &mut ltx)?;

    ltx.write_to(&mut file)?;

    let source: Ltx = Ltx::read_from_path(config_path)?;

    assert_eq!(AlifeObjectActor::import("first", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectActor = AlifeObjectActor {
      base: AlifeObjectCreature {
        base: AlifeObjectDynamicVisual {
          base: AlifeObjectAbstract {
            game_vertex_id: 8,
            distance: 594.25,
            direct_control: 367,
            level_vertex_id: 253,
            flags: 155,
            custom_data: String::from("custom-data"),
            story_id: 5346,
            spawn_story_id: 254,
          },
          visual_name: String::from("visual-name"),
          visual_flags: 235,
        },
        team: 1,
        squad: 67,
        group: 66,
        health: 1.0,
        dynamic_out_restrictions: vec![1, 2, 3, 4],
        dynamic_in_restrictions: vec![5, 6, 7, 8],
        killer_id: 6532,
        game_death_time: 0,
      },
      trader: AlifeObjectTraderAbstract {
        money: 56255,
        specific_character: String::from("specific-character"),
        trader_flags: 37,
        character_profile: String::from("character-profile"),
        community_index: 5,
        rank: 6,
        reputation: 3,
        character_name: String::from("character-name"),
        dead_body_can_take: 0,
        dead_body_closed: 1,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 35,
        source_id: 67,
      },
      holder_id: 0,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectActor>(&serialized)?, original);

    Ok(())
  }
}
