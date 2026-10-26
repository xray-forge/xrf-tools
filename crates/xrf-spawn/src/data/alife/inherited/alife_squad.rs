use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

use crate::data::alife::inherited::alife_object_online_offline_group::AlifeObjectOnlineOfflineGroup;
use crate::data::generic::script_save_marker::ScriptSaveMarker;
use crate::data::generic::script_to_string::ScriptToString;

/// A squad as its script class saves one, `sim_squad_scripted` in vanilla and `Squad` in xrf-engine: both follow the
/// group's fields with four `tostring` strings closed by a save marker - its target, its respawn point and the section
/// that point spawns, and its smart terrain - and add nothing to the update packet.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeSquad {
  pub base: AlifeObjectOnlineOfflineGroup,
  /// The object the squad heads for, `current_target_id`.
  pub current_target_id: Option<u32>,
  /// The smart terrain that respawns it, `respawn_point_id`.
  pub respawn_point_id: Option<u32>,
  /// The respawn section it was spawned from, `respawn_point_prop_section`.
  pub respawn_point_section: Option<String>,
  /// The smart terrain it is assigned to, `smart_id` (`assignedTerrainId` in xrf-engine).
  pub assigned_terrain_id: Option<u32>,
}

impl AlifeSquad {
  /// The script class the trailing fields belong to, named in a refused save marker.
  const SCRIPT_CLASS: &'static str = "sim_squad_scripted";

  /// What each refused id is named as.
  const TARGET: &'static str = "Squad target";
  const RESPAWN_POINT: &'static str = "Squad respawn point";
  const ASSIGNED_TERRAIN: &'static str = "Squad smart terrain";
}

impl ChunkReadWrite for AlifeSquad {
  /// Read the squad from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let base: AlifeObjectOnlineOfflineGroup = reader.read_xr::<T, _>()?;

    ScriptSaveMarker::read::<T, _, _>(reader, Self::SCRIPT_CLASS, |reader| {
      Ok(Self {
        base,
        current_target_id: ScriptToString::parse_id(&reader.read_w1251_string()?, Self::TARGET)?,
        respawn_point_id: ScriptToString::parse_id(&reader.read_w1251_string()?, Self::RESPAWN_POINT)?,
        respawn_point_section: ScriptToString::parse_text(reader.read_w1251_string()?),
        assigned_terrain_id: ScriptToString::parse_id(&reader.read_w1251_string()?, Self::ASSIGNED_TERRAIN)?,
      })
    })
  }

  /// Write the squad into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;

    ScriptSaveMarker::write::<T>(writer, Self::SCRIPT_CLASS, |writer| {
      writer.write_w1251_string(&ScriptToString::format_id(self.current_target_id))?;
      writer.write_w1251_string(&ScriptToString::format_id(self.respawn_point_id))?;
      writer.write_w1251_string(ScriptToString::format_text(self.respawn_point_section.as_deref()))?;
      writer.write_w1251_string(&ScriptToString::format_id(self.assigned_terrain_id))?;

      Ok(())
    })
  }
}

impl LtxImportExport for AlifeSquad {
  /// Import the squad from ltx file section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectOnlineOfflineGroup::import(section_name, ltx)?,
      current_target_id: ScriptToString::parse_id(
        &section.read_field::<String>("squad.current_target_id")?,
        Self::TARGET,
      )?,
      respawn_point_id: ScriptToString::parse_id(
        &section.read_field::<String>("squad.respawn_point_id")?,
        Self::RESPAWN_POINT,
      )?,
      respawn_point_section: ScriptToString::parse_text(section.read_field("squad.respawn_point_section")?),
      assigned_terrain_id: ScriptToString::parse_id(
        &section.read_field::<String>("squad.assigned_terrain_id")?,
        Self::ASSIGNED_TERRAIN,
      )?,
    })
  }

  /// Export object data into ltx file, each value as the string the script writes for it.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set(
        "squad.current_target_id",
        ScriptToString::format_id(self.current_target_id),
      )
      .set(
        "squad.respawn_point_id",
        ScriptToString::format_id(self.respawn_point_id),
      )
      .set(
        "squad.respawn_point_section",
        ScriptToString::format_text(self.respawn_point_section.as_deref()),
      )
      .set(
        "squad.assigned_terrain_id",
        ScriptToString::format_id(self.assigned_terrain_id),
      );

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};

  use crate::data::alife::inherited::alife_object_online_offline_group::AlifeObjectOnlineOfflineGroup;
  use crate::data::alife::inherited::alife_squad::AlifeSquad;

  fn new_object() -> AlifeSquad {
    AlifeSquad {
      base: AlifeObjectOnlineOfflineGroup::new_mock(vec![]),
      current_target_id: None,
      respawn_point_id: Some(812),
      respawn_point_section: Some(String::from("spawn_stalker")),
      assigned_terrain_id: None,
    }
  }

  /// Four strings after the group, closed by the marker counting them.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeSquad = new_object();
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(&writer.buffer[31..], b"nil\0812\0spawn_stalker\0nil\0\x1a\0".as_slice());

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(AlifeSquad::read::<XRayByteOrder, _>(&mut reader)?, original);
    reader.assert_read("Expect the squad to be read whole")?;

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeSquad = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeSquad::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeSquad = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeSquad>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
