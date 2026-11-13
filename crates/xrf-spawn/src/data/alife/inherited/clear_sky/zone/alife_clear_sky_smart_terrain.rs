use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxWrite};

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_smart_zone::AlifeSmartZone;
use crate::data::alife::inherited::clear_sky::zone::clear_sky_combat_manager::ClearSkyCombatManager;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::generic::script_absent_state::ScriptAbsentState;
use crate::data::generic::script_save_marker::ScriptSaveMarker;

/// A smart terrain as Clear Sky's `se_smart_terrain` saves one (`gamedata-cs/scripts/smart_terrain.script`): the smart
/// zone's spawn fields, then, closed by a save marker, its combat manager and the stalkers' jobs and the jobs' death
/// times. The jobs and death times are records only a running simulation fills; a spawn holds none, so their counts are
/// read as empty and any other value is refused rather than read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeClearSkySmartTerrain {
  pub base: AlifeSmartZone,
  pub combat_manager: ClearSkyCombatManager,
}

impl AlifeClearSkySmartTerrain {
  /// The script class the trailing fields belong to, named in a refusal.
  const SCRIPT_CLASS: &'static str = "se_smart_terrain";

  /// The prefix of the combat manager's LTX keys.
  const COMBAT_PREFIX: &'static str = "smart_terrain.combat";
}

impl AlifeStateReadWrite for AlifeClearSkySmartTerrain {
  /// Read the terrain's spawn fields. Its `STATE_Read` returns before its own under the Level Editor, so the editor's
  /// form ending at the smart zone takes the combat manager's constructor state.
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      combat_manager: ScriptSaveMarker::read_or_constructed::<T, _, _>(
        reader,
        Self::SCRIPT_CLASS,
        ClearSkyCombatManager::default,
        |reader| {
          let combat_manager: ClearSkyCombatManager = reader.read_xr::<T, _>()?;

          ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "stalkers' jobs count", 0)?;
          ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "jobs' death times count", 0)?;

          Ok(combat_manager)
        },
      )?,
    })
  }

  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    writer.write_xr::<T, _>(&self.base)?;

    ScriptSaveMarker::write::<T, _>(writer, Self::SCRIPT_CLASS, |writer| {
      writer.write_xr::<T, _>(&self.combat_manager)?;
      // No stalkers' jobs and no jobs' death times.
      writer.write_u8(0)?;
      writer.write_u8(0)
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
      combat_manager: ClearSkyCombatManager::import(section, Self::COMBAT_PREFIX)?,
    })
  }

  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, _: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.combat_manager.export(section_name, Self::COMBAT_PREFIX, ltx);

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
  use crate::data::alife::inherited::clear_sky::zone::alife_clear_sky_smart_terrain::AlifeClearSkySmartTerrain;
  use crate::data::alife::inherited::clear_sky::zone::clear_sky_combat_manager::ClearSkyCombatManager;
  use crate::data::alife::spawn_layout::SpawnLayout;
  use crate::data::generic::shape::Shape;

  fn new_terrain() -> AlifeClearSkySmartTerrain {
    AlifeClearSkySmartTerrain {
      base: AlifeSmartZone {
        base: AlifeObjectSpaceRestrictor {
          base: AlifeObjectAbstract {
            game_vertex_id: 1002,
            distance: 65.25,
            direct_control: 1,
            level_vertex_id: 3213,
            flags: 62,
            custom_data: String::from("[smart_terrain]\r\nsquad_id = 31"),
            story_id: u32::MAX,
            spawn_story_id: u32::MAX,
          },
          shape: vec![Shape::Sphere(Sphere {
            center: Vector3d::new(0.0, 0.0, 0.0),
            radius: 3.1,
          })],
          restrictor_type: 3,
        },
      },
      combat_manager: ClearSkyCombatManager::default(),
    }
  }

  /// The terrain's own fields: the combat manager, two empty counts and the marker counting all of them.
  #[test]
  fn writes_the_tail_closed_by_its_marker() -> XrfResult {
    let original: AlifeClearSkySmartTerrain = new_terrain();
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write_state::<XRayByteOrder, _>(&mut writer, SpawnLayout::CLEAR_SKY)?;

    assert_eq!(writer.buffer[writer.buffer.len() - 4..], [0, 0, 23, 0]);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      AlifeClearSkySmartTerrain::read_state::<XRayByteOrder, _>(&mut reader, SpawnLayout::CLEAR_SKY)?,
      original
    );
    assert!(reader.is_ended());

    Ok(())
  }

  /// A stalker holding a job would be followed by its record, which is not read.
  #[test]
  fn refuses_stalkers_holding_jobs() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    new_terrain().write_state::<XRayByteOrder, _>(&mut writer, SpawnLayout::CLEAR_SKY)?;

    let jobs_at: usize = writer.buffer.len() - 4;

    writer.buffer[jobs_at] = 2;

    let error: String = AlifeClearSkySmartTerrain::read_state::<XRayByteOrder, _>(
      &mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(&writer.buffer)?,
      SpawnLayout::CLEAR_SKY,
    )
    .expect_err("two stalkers holding jobs")
    .to_string();

    assert!(
      error.contains("se_smart_terrain holds stalkers' jobs count 2"),
      "{error}"
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeClearSkySmartTerrain = new_terrain();
    let mut ltx: Ltx = Ltx::new();

    original.combat_manager.see_actor_enemy = Some(String::from("false"));
    original.export_state("data", &mut ltx, SpawnLayout::CLEAR_SKY)?;

    assert_eq!(
      AlifeClearSkySmartTerrain::import_state("data", &ltx, SpawnLayout::CLEAR_SKY)?,
      original
    );

    Ok(())
  }
}
