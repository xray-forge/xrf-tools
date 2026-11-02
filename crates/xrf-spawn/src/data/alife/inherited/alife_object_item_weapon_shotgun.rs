use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_utils::{vector_from_string, vector_to_string};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined;

/// A shotgun, `CSE_ALifeItemWeaponShotGun`, which the engine derives from the magazined weapon.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemWeaponShotgun {
  pub base: AlifeObjectItemWeaponMagazined,
  /// The ammo type of each shell loaded, `m_AmmoIDs`, behind a byte counting them.
  pub update_ammo_ids: Vec<u8>,
}

impl AlifePacketsReadWrite for AlifeObjectItemWeaponShotgun {
  /// Read a shotgun from both packets; the shells loaded end its update packet.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    let base: AlifeObjectItemWeaponMagazined = AlifeObjectItemWeaponMagazined::read::<T, _>(spawn, update)?;
    let count: u8 = update.read_u8()?;

    Ok(Self {
      base,
      update_ammo_ids: update.read_bytes(count.into())?,
    })
  }

  /// Write a shotgun into both packets.
  ///
  /// # Errors
  ///
  /// Refuses more shells than the count's byte holds.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    let count: u8 = u8::try_from(self.update_ammo_ids.len()).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "Shotgun loads {} shells, more than the byte counting them holds",
        self.update_ammo_ids.len()
      ))
    })?;

    self.base.write::<T>(spawn, update)?;
    update.write_u8(count)?;
    update.write_all(&self.update_ammo_ids)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemWeaponShotgun {
  /// Import ALife object data from ltx config section.
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
      update_ammo_ids: vector_from_string(&section.read_field::<String>("item_weapon_shotgun.update_ammo_ids")?)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx.with_section(section_name).set(
      "item_weapon_shotgun.update_ammo_ids",
      vector_to_string(&self.update_ammo_ids),
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
  use crate::data::alife::inherited::alife_object_item_weapon_shotgun::AlifeObjectItemWeaponShotgun;

  fn new_object() -> AlifeObjectItemWeaponShotgun {
    AlifeObjectItemWeaponShotgun {
      base: AlifeObjectItemWeaponMagazined::new_mock(Vector3d::new(1.0, 2.0, 3.0)),
      update_ammo_ids: vec![0, 1, 1],
    }
  }

  /// The magazined weapon's fire mode, then a count of shells and each one's ammo type.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemWeaponShotgun = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (60, 63 + 1 + 3, original));

    Ok(())
  }

  /// An empty shotgun without a physics state is the eleven-byte update packet placed ones carry.
  #[test]
  fn reads_an_empty_shotgun() -> XrfResult {
    let mut original: AlifeObjectItemWeaponShotgun = new_object();

    original.base.base.base.update_state = None;
    original.update_ammo_ids.clear();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (60, 11, original));

    Ok(())
  }

  #[test]
  fn refuses_more_shells_than_a_byte_counts() {
    let mut original: AlifeObjectItemWeaponShotgun = new_object();

    original.update_ammo_ids = vec![0; 256];

    let error: String = original
      .read_back::<XRayByteOrder>()
      .expect_err("256 shells")
      .to_string();

    assert!(error.contains("loads 256 shells"), "{error}");
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemWeaponShotgun = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemWeaponShotgun::import("data", &ltx)?, original);

    Ok(())
  }
}
