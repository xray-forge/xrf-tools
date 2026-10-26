use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_utils::{decode_bytes_from_base64, encode_bytes_to_base64};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
use crate::data::generic::script_save_marker::ScriptSaveMarker;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeActor {
  pub base: AlifeObjectActor,
  /// The script class's own field, `se_actor` in vanilla.
  pub start_position_filled: u8,
  /// The update packet, carried as stored. The engine reads it over the spawn packet, so the actor starts where it
  /// says.
  // todo: Read the creature and actor update fields, `CSE_ALifeCreatureActor::UPDATE_Read`, in place of these bytes.
  pub update_data: Vec<u8>,
}

impl AlifeActor {
  /// The script class the trailing field belongs to, named in a refused save marker.
  const SCRIPT_CLASS: &'static str = "se_actor";
}

#[cfg(test)]
impl AlifeActor {
  /// An actor as a new game places one, with no physics state saved.
  pub fn new_mock() -> Self {
    Self {
      base: AlifeObjectActor {
        base: AlifeObjectCreature {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 621,
              distance: 0.0,
              direct_control: 1,
              level_vertex_id: 52235,
              flags: 0,
              custom_data: String::new(),
              story_id: AlifeObjectAbstract::INVALID_STORY_ID,
              spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
            },
            visual_name: String::from("actors\\stalker_hero\\stalker_hero_1"),
            visual_flags: 0,
          },
          team: 0,
          squad: 0,
          group: 0,
          health: 1.0,
          dynamic_out_restrictions: Vec::new(),
          dynamic_in_restrictions: Vec::new(),
          killer_id: u16::MAX,
          game_death_time: 0,
        },
        trader: AlifeObjectTraderAbstract {
          money: 5000,
          specific_character: String::from("actor"),
          trader_flags: 0,
          character_profile: String::from("actor"),
          community_index: 0,
          rank: 0,
          reputation: 0,
          character_name: String::new(),
          dead_body_can_take: 1,
          dead_body_closed: 0,
        },
        skeleton: AlifeObjectSkeleton {
          name: String::new(),
          flags: 0,
          source_id: u16::MAX,
        },
        holder_id: u16::MAX,
      },
      start_position_filled: 0,
      update_data: Vec::new(),
    }
  }
}

impl AlifePacketsReadWrite for AlifeActor {
  /// Read actor data from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: spawn.read_xr::<T, _>()?,
      start_position_filled: ScriptSaveMarker::read::<T, _, _>(spawn, Self::SCRIPT_CLASS, |reader| {
        Ok(reader.read_u8()?)
      })?,
      update_data: update.read_remaining()?,
    })
  }

  /// Write actor data into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    spawn.write_xr::<T, _>(&self.base)?;

    ScriptSaveMarker::write::<T>(spawn, Self::SCRIPT_CLASS, |writer| {
      writer.write_u8(self.start_position_filled)?;

      Ok(())
    })?;

    update.write_all(&self.update_data)?;

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
      start_position_filled: section.read_field("actor.start_position_filled")?,
      update_data: decode_bytes_from_base64(&section.read_field::<String>("actor.update_data")?)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("actor.start_position_filled", self.start_position_filled.to_string())
      .set("actor.update_data", encode_bytes_to_base64(&self.update_data));

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_test_utils::GeneratedResources;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_actor::AlifeActor;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;
  use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;

  #[test]
  fn test_read_write() -> XrfResult {
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
      update_data: vec![0, 1, 2, 3],
    };

    assert_eq!(original.read_back::<XRayByteOrder>()?, (196, 4, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
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
      update_data: vec![0, 1, 2, 3],
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

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
      update_data: vec![0, 1, 2, 3],
    };

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeActor>(&serialized)?, original);

    Ok(())
  }
}
