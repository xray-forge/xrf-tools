use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_smart_zone::AlifeSmartZone;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::generic::script_absent_state::ScriptAbsentState;
use crate::data::generic::script_save_marker::ScriptSaveMarker;

/// A faction as Clear Sky's `se_sim_faction` saves one (`gamedata-cs/scripts/sim_faction.script`), the class Clear Sky
/// registers for `SFACTION`: the smart zone's spawn fields, then, closed by a save marker, whether the player leads it,
/// whether its start position was filled and its expansion level, followed by its last spawn time and four tables.
///
/// The last spawn time, the squads' targets, the random tasks, the attack counts and the squads are what a running
/// simulation fills; a spawn holds `nil` and four empty tables, so those are read as such and any other value is refused
/// rather than read. The time is `utils.w_CTime`'s Clear Sky form, whose `nil` is a lone 255.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeClearSkySimFaction {
  pub base: AlifeSmartZone,
  /// `community_player`, a `w_bool`.
  pub community_player: u8,
  /// `start_position_filled`, a `w_bool`.
  pub start_position_filled: u8,
  pub current_expansion_level: u8,
}

impl AlifeClearSkySimFaction {
  /// The script class the trailing fields belong to, named in a refusal.
  const SCRIPT_CLASS: &'static str = "se_sim_faction";

  /// `utils.w_CTime`'s byte for a `nil` time.
  const NIL_TIME: u8 = u8::MAX;

  /// What the script class's constructor holds, which the Level Editor's form takes in place of saved fields.
  const CONSTRUCTED: [u8; 3] = [0, 0, 0];
}

impl AlifeStateReadWrite for AlifeClearSkySimFaction {
  /// Read the faction's spawn fields. Its `STATE_Read` returns before its own under the Level Editor, so the editor's
  /// form ending at the smart zone takes the constructor's values.
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    let base: AlifeSmartZone = reader.read_xr::<T, _>()?;
    let [community_player, start_position_filled, current_expansion_level] =
      ScriptSaveMarker::read_or_constructed::<T, _, _>(
        reader,
        Self::SCRIPT_CLASS,
        || Self::CONSTRUCTED,
        |reader| {
          let fields: [u8; 3] = [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?];

          ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "last spawn time", Self::NIL_TIME)?;
          ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "squads' targets count", 0)?;
          ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "random tasks count", 0)?;
          ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "attack counts count", 0)?;
          ScriptAbsentState::read_u16::<T, _>(reader, Self::SCRIPT_CLASS, "squads count", 0)?;

          Ok(fields)
        },
      )?;

    Ok(Self {
      base,
      community_player,
      start_position_filled,
      current_expansion_level,
    })
  }

  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    writer.write_xr::<T, _>(&self.base)?;

    ScriptSaveMarker::write::<T, _>(writer, Self::SCRIPT_CLASS, |writer| {
      writer.write_u8(self.community_player)?;
      writer.write_u8(self.start_position_filled)?;
      writer.write_u8(self.current_expansion_level)?;
      writer.write_u8(Self::NIL_TIME)?;
      // No squads' targets, random tasks, attack counts or squads.
      writer.write_u8(0)?;
      writer.write_u8(0)?;
      writer.write_u8(0)?;
      writer.write_u16::<T>(0)
    })
  }

  fn import_state<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeSmartZone::import(section_name, ltx)?,
      community_player: section.read_field("sim_faction.community_player")?,
      start_position_filled: section.read_field("sim_faction.start_position_filled")?,
      current_expansion_level: section.read_field("sim_faction.current_expansion_level")?,
    })
  }

  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, _: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("sim_faction.community_player", self.community_player)
      .set("sim_faction.start_position_filled", self.start_position_filled)
      .set("sim_faction.current_expansion_level", self.current_expansion_level);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::{Sphere, Vector3d};

  use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::alife::inherited::alife_smart_zone::AlifeSmartZone;
  use crate::data::alife::inherited::clear_sky::zone::alife_clear_sky_sim_faction::AlifeClearSkySimFaction;
  use crate::data::alife::spawn_layout::SpawnLayout;
  use crate::data::generic::shape::Shape;

  fn new_faction() -> AlifeClearSkySimFaction {
    AlifeClearSkySimFaction {
      base: AlifeSmartZone {
        base: AlifeObjectSpaceRestrictor {
          base: AlifeObjectAbstract {
            game_vertex_id: u16::MAX,
            distance: 0.0,
            direct_control: 1,
            level_vertex_id: u32::MAX,
            flags: 62,
            custom_data: String::from("[faction]\r\nname = killer"),
            story_id: u32::MAX,
            spawn_story_id: u32::MAX,
          },
          shape: vec![Shape::Sphere(Sphere {
            center: Vector3d::new(0.0, 0.0, 0.0),
            radius: 1.0,
          })],
          restrictor_type: 3,
        },
      },
      community_player: 0,
      start_position_filled: 0,
      current_expansion_level: 0,
    }
  }

  /// The faction's own fields: three bytes, the `nil` time, four empty tables and the marker counting them.
  #[test]
  fn writes_the_tail_closed_by_its_marker() -> XrfResult {
    let original: AlifeClearSkySimFaction = new_faction();
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write_state::<XRayByteOrder, _>(&mut writer, SpawnLayout::CLEAR_SKY)?;

    assert_eq!(
      writer.buffer[writer.buffer.len() - 11..],
      [0, 0, 0, 255, 0, 0, 0, 0, 0, 9, 0]
    );

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      AlifeClearSkySimFaction::read_state::<XRayByteOrder, _>(&mut reader, SpawnLayout::CLEAR_SKY)?,
      original
    );
    assert!(reader.is_ended());

    Ok(())
  }

  /// A faction that last spawned at some time would carry it in a form that is not read.
  #[test]
  fn refuses_a_last_spawn_time() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    new_faction().write_state::<XRayByteOrder, _>(&mut writer, SpawnLayout::CLEAR_SKY)?;

    let time_at: usize = writer.buffer.len() - 8;

    writer.buffer[time_at] = 0;

    let error: String = AlifeClearSkySimFaction::read_state::<XRayByteOrder, _>(
      &mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(&writer.buffer)?,
      SpawnLayout::CLEAR_SKY,
    )
    .expect_err("a zero time")
    .to_string();

    assert!(error.contains("se_sim_faction holds last spawn time 0"), "{error}");

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeClearSkySimFaction = new_faction();
    let mut ltx: Ltx = Ltx::new();

    original.current_expansion_level = 3;
    original.export_state("data", &mut ltx, SpawnLayout::CLEAR_SKY)?;

    assert_eq!(
      AlifeClearSkySimFaction::import_state("data", &ltx, SpawnLayout::CLEAR_SKY)?,
      original
    );

    Ok(())
  }
}
