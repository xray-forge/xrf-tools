use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemWeaponMagazinedWgl {
  pub base: AlifeObjectItemWeaponMagazined,
  /// `m_bGrenadeMode`, nonzero while the launcher is selected.
  pub update_grenade_mode: u8,
}

impl AlifePacketsReadWrite for AlifeObjectItemWeaponMagazinedWgl {
  /// Read a magazined weapon with a launcher from both packets. Its grenade mode opens the update packet, ahead of
  /// every field it inherits, as `CSE_ALifeItemWeaponMagazinedWGL::UPDATE_Read` takes it first.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    let update_grenade_mode: u8 = update.read_u8()?;

    Ok(Self {
      base: AlifeObjectItemWeaponMagazined::read::<T, _>(spawn, update)?,
      update_grenade_mode,
    })
  }

  /// Write a magazined weapon with a launcher into both packets, its grenade mode first in the update packet.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    update.write_u8(self.update_grenade_mode)?;

    self.base.write::<T>(spawn, update)
  }
}

impl LtxImportExport for AlifeObjectItemWeaponMagazinedWgl {
  /// Read magazined weapon with launcher from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectItemWeaponMagazined::import(section_name, ltx)?,
      update_grenade_mode: section.read_field("item_weapon_magazined_wgl.update_grenade_mode")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx.with_section(section_name).set(
      "item_weapon_magazined_wgl.update_grenade_mode",
      self.update_grenade_mode.to_string(),
    );

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined;
  use crate::data::alife::inherited::alife_object_item_weapon_magazined_wgl::AlifeObjectItemWeaponMagazinedWgl;

  fn new_object() -> AlifeObjectItemWeaponMagazinedWgl {
    AlifeObjectItemWeaponMagazinedWgl {
      base: AlifeObjectItemWeaponMagazined::new_mock(Vector3d::new(1.0, 2.0, 3.0)),
      update_grenade_mode: 1,
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemWeaponMagazinedWgl = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (60, 64, original));

    Ok(())
  }

  /// The grenade mode precedes the physics state, so the state's opening byte is the update packet's second.
  #[test]
  fn writes_the_grenade_mode_ahead_of_the_physics_state() -> XrfResult {
    let original: AlifeObjectItemWeaponMagazinedWgl = new_object();
    let mut spawn: xrf_chunk::ChunkWriter = xrf_chunk::ChunkWriter::new();
    let mut update: xrf_chunk::ChunkWriter = xrf_chunk::ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut spawn, &mut update)?;

    // Grenade mode, then ten items, enabled, with both velocities left out.
    assert_eq!(update.buffer[..2], [1, 0b1110_1010]);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemWeaponMagazinedWgl = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemWeaponMagazinedWgl::import("data", &ltx)?, original);

    Ok(())
  }
}
