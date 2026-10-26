use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
#[cfg(test)]
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemWeaponMagazined {
  pub base: AlifeObjectItemWeapon,
  /// The fire mode selected, `m_u8CurFireMode`.
  pub update_fire_mode: u8,
}

impl AlifePacketsReadWrite for AlifeObjectItemWeaponMagazined {
  /// Read a magazined weapon from both packets; its fire mode ends the update packet's weapon fields.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItemWeapon::read::<T, _>(spawn, update)?,
      update_fire_mode: update.read_u8()?,
    })
  }

  /// Write a magazined weapon into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;
    update.write_u8(self.update_fire_mode)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemWeaponMagazined {
  /// Import a magazined weapon from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectItemWeapon::import(section_name, ltx)?,
      update_fire_mode: section.read_field("item_weapon_magazined.update_fire_mode")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx.with_section(section_name).set(
      "item_weapon_magazined.update_fire_mode",
      self.update_fire_mode.to_string(),
    );

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectItemWeaponMagazined {
  /// A magazined weapon placed at `position`, firing in its second mode.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectItemWeapon::new_mock(position),
      update_fire_mode: 1,
    }
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

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemWeaponMagazined =
      AlifeObjectItemWeaponMagazined::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(original.read_back::<XRayByteOrder>()?, (60, 63, original));

    Ok(())
  }

  /// Without a physics state a magazined weapon's update packet is the ten bytes most placed ones carry.
  #[test]
  fn reads_a_weapon_without_a_physics_state() -> XrfResult {
    let mut original: AlifeObjectItemWeaponMagazined = AlifeObjectItemWeaponMagazined::new_mock(Vector3d::ZERO);

    original.base.base.update_state = None;

    assert_eq!(original.read_back::<XRayByteOrder>()?, (60, 10, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemWeaponMagazined =
      AlifeObjectItemWeaponMagazined::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemWeaponMagazined::import("data", &ltx)?, original);

    Ok(())
  }
}
