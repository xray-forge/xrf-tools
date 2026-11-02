use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_utils::vector_to_string;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::physics::raw_float::RawFloat;

/// A stationary machine gun, `CSE_ALifeStationaryMgun`: a visual whose update packet carries whether it fires and
/// where it aims. The engine leaves both uninitialised in a new gun, so the aim keeps the bits it is stored in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectStationaryMgun {
  pub base: AlifeObjectDynamicVisual,
  /// Whether it fires, `m_bWorking`, one byte.
  pub update_working: u8,
  /// The direction it aims at its enemy in, `m_destEnemyDir`.
  pub update_enemy_direction: [RawFloat; 3],
}

impl AlifePacketsReadWrite for AlifeObjectStationaryMgun {
  /// Read the gun from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: spawn.read_xr::<T, _>()?,
      update_working: update.read_u8()?,
      update_enemy_direction: RawFloat::read_array::<T, D, 3>(update)?,
    })
  }

  /// Write the gun into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    spawn.write_xr::<T, _>(&self.base)?;

    update.write_u8(self.update_working)?;

    RawFloat::write_array::<T>(update, &self.update_enemy_direction)
  }
}

impl LtxImportExport for AlifeObjectStationaryMgun {
  /// Import the gun from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      update_working: section.read_field("stationary_mgun.update_working")?,
      update_enemy_direction: RawFloat::import_array(section, "stationary_mgun.update_enemy_direction")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("stationary_mgun.update_working", self.update_working.to_string())
      .set(
        "stationary_mgun.update_enemy_direction",
        vector_to_string(&self.update_enemy_direction),
      );

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_stationary_mgun::AlifeObjectStationaryMgun;
  use crate::data::physics::raw_float::RawFloat;

  /// A gun as the game writes a new one: its working byte and aim never initialised.
  fn new_object() -> AlifeObjectStationaryMgun {
    AlifeObjectStationaryMgun {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 621,
          distance: 0.0,
          direct_control: 1,
          level_vertex_id: 52235,
          flags: 0xffff_ffbf,
          custom_data: String::new(),
          story_id: AlifeObjectAbstract::INVALID_STORY_ID,
          spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
        },
        visual_name: String::from("dynamics\\weapons\\wpn_mounted\\wpn_mounted_pkm_trenoga"),
        visual_flags: 0,
      },
      update_working: 1,
      update_enemy_direction: [RawFloat(0x0000_04db), RawFloat(0x3979_8331), RawFloat(0x7fc0_0001)],
    }
  }

  /// A byte and three floats after the visual's empty update.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectStationaryMgun = new_object();
    let (spawn_size, update_size, read) = original.read_back::<XRayByteOrder>()?;

    assert_eq!(
      (spawn_size, update_size, read),
      (27 + original.base.visual_name.len() + 1 + 1, 13, original)
    );

    Ok(())
  }

  /// The uninitialised aim survives text and JSON with its bits, a NaN's payload included.
  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectStationaryMgun = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectStationaryMgun::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectStationaryMgun = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectStationaryMgun>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
