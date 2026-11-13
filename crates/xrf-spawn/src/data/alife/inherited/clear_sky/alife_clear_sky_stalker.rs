use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_human_stalker::AlifeObjectHumanStalker;
use crate::data::alife::inherited::clear_sky::clear_sky_job_online::ClearSkyJobOnline;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
use crate::data::generic::script_to_string::ScriptToString;

/// A stalker as Clear Sky's `se_stalker` saves one (`gamedata-cs/scripts/se_stalker.script`): the server class's spawn
/// fields, then whether its job keeps it online, whether its corpse's loot was dropped, the squad it belongs to and
/// whether the simulation forces it online, unmarked, adding nothing to the update packet.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeClearSkyStalker {
  pub base: AlifeObjectHumanStalker,
  pub job_online: ClearSkyJobOnline,
  /// Whether the death manager already dropped the corpse's loot, `w_bool` of `death_droped`; zero for a new object.
  pub loot_dropped: u8,
  /// `squad_id` as the script writes it, `tostring` of the squad's id; `None` for `nil`.
  pub squad_id: Option<String>,
  /// `sim_forced_online`, a `w_bool`.
  pub sim_forced_online: u8,
}

impl AlifeClearSkyStalker {
  /// The script class the trailing fields belong to, named in a refusal.
  const SCRIPT_CLASS: &'static str = "se_stalker";

  /// Place the stalker at `position`, facing `direction`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.base.move_to(position, direction);
  }
}

#[cfg(test)]
impl AlifeClearSkyStalker {
  /// A stalker at `position` as a Clear Sky level spawns one.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectHumanStalker::new_mock(position),
      job_online: ClearSkyJobOnline::Unset,
      loot_dropped: 0,
      squad_id: None,
      sim_forced_online: 0,
    }
  }
}

impl AlifePacketsReadWrite for AlifeClearSkyStalker {
  fn set_spawned_update(&mut self, header: &SpawnPacketHeader) {
    self.base.set_spawned_update(header);
  }

  /// Read the stalker from both packets; the script class adds to the spawn packet alone.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self> {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    Ok(Self {
      base: AlifeObjectHumanStalker::read::<T, _, _>(spawn, update, layout)?,
      job_online: spawn.read_xr::<T, _>()?,
      loot_dropped: spawn.read_u8()?,
      squad_id: ScriptToString::parse_text(spawn.read_w1251_string()?),
      sim_forced_online: spawn.read_u8()?,
    })
  }

  /// Write the stalker into both packets.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    self.base.write::<T, _, _>(spawn, update, layout)?;

    spawn.write_xr::<T, _>(&self.job_online)?;
    spawn.write_u8(self.loot_dropped)?;
    spawn.write_w1251_string(ScriptToString::format_text(self.squad_id.as_deref()))?;
    spawn.write_u8(self.sim_forced_online)
  }

  /// Import stalker data from ltx file section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectHumanStalker::import(section_name, ltx, layout)?,
      job_online: ClearSkyJobOnline::import(section, "stalker")?,
      loot_dropped: section.read_field("stalker.loot_dropped")?,
      squad_id: ScriptToString::parse_text(section.read_field("stalker.squad_id")?),
      sim_forced_online: section.read_field("stalker.sim_forced_online")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx, layout)?;
    self.job_online.export(section_name, "stalker", ltx);

    ltx
      .with_section(section_name)
      .set("stalker.loot_dropped", self.loot_dropped)
      .set(
        "stalker.squad_id",
        ScriptToString::format_text(self.squad_id.as_deref()),
      )
      .set("stalker.sim_forced_online", self.sim_forced_online);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::clear_sky::alife_clear_sky_stalker::AlifeClearSkyStalker;
  use crate::data::alife::inherited::clear_sky::clear_sky_job_online::ClearSkyJobOnline;
  use crate::data::alife::spawn_layout::SpawnLayout;

  /// A level's stalker: the job byte for `nil`, the loot flag, the squad as `nil` and the forced flag close the spawn
  /// packet.
  #[test]
  fn writes_the_tail_in_the_script_order() -> XrfResult {
    let original: AlifeClearSkyStalker = AlifeClearSkyStalker::new_mock(Vector3d::new(-12.5, 0.75, 310.0));
    let (base_spawn_size, base_update_size, _) = original.base.read_back::<XRayByteOrder>(SpawnLayout::CLEAR_SKY)?;
    let mut spawn: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder, _, _>(&mut spawn, &mut ChunkWriter::new(), SpawnLayout::CLEAR_SKY)?;

    assert_eq!(&spawn.buffer[base_spawn_size..], b"\x02\0nil\0\0");
    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CLEAR_SKY)?,
      (base_spawn_size + 7, base_update_size, original)
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeClearSkyStalker = AlifeClearSkyStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.job_online = ClearSkyJobOnline::Offline;
    original.loot_dropped = 1;
    original.squad_id = Some(String::from("4"));
    original.export("data", &mut ltx, SpawnLayout::CLEAR_SKY)?;

    assert_eq!(
      AlifeClearSkyStalker::import("data", &ltx, SpawnLayout::CLEAR_SKY)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeClearSkyStalker = AlifeClearSkyStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(
      serde_json::from_str::<AlifeClearSkyStalker>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
