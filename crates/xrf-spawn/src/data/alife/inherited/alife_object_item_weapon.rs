use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
#[cfg(test)]
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

/// A weapon, `CSE_ALifeItemWeapon`. Its update packet repeats the rounds loaded, weapon state, addons and ammo type the
/// spawn packet stores, and the engine reads them over the spawn packet's, so those are what the game starts with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemWeapon {
  pub base: AlifeObjectItem,
  pub ammo_current: u16,
  pub ammo_elapsed: u16,
  pub weapon_state: u8,
  pub addon_flags: u8,
  pub ammo_type: u8,
  pub elapsed_grenades: u8,
  /// The condition as the update packet quantises it, which the engine reads over the spawn packet's.
  pub update_condition: u8,
  pub update_weapon_flags: u8,
  pub update_ammo_elapsed: u16,
  pub update_addon_flags: u8,
  pub update_ammo_type: u8,
  pub update_weapon_state: u8,
  /// `m_bZoom`, nonzero while aiming.
  pub update_zoom: u8,
}

impl AlifeObjectItemWeapon {
  /// The condition the engine reads from the update packet.
  pub fn get_update_condition(&self) -> f32 {
    AlifeObjectItem::dequantize_condition(self.update_condition)
  }
}

impl AlifePacketsReadWrite for AlifeObjectItemWeapon {
  /// Read a weapon from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _>(spawn, update)?,
      ammo_current: spawn.read_u16::<T>()?,
      ammo_elapsed: spawn.read_u16::<T>()?,
      weapon_state: spawn.read_u8()?,
      addon_flags: spawn.read_u8()?,
      ammo_type: spawn.read_u8()?,
      elapsed_grenades: spawn.read_u8()?,
      update_condition: update.read_u8()?,
      update_weapon_flags: update.read_u8()?,
      update_ammo_elapsed: update.read_u16::<T>()?,
      update_addon_flags: update.read_u8()?,
      update_ammo_type: update.read_u8()?,
      update_weapon_state: update.read_u8()?,
      update_zoom: update.read_u8()?,
    })
  }

  /// Write a weapon into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;

    spawn.write_u16::<T>(self.ammo_current)?;
    spawn.write_u16::<T>(self.ammo_elapsed)?;
    spawn.write_u8(self.weapon_state)?;
    spawn.write_u8(self.addon_flags)?;
    spawn.write_u8(self.ammo_type)?;
    spawn.write_u8(self.elapsed_grenades)?;

    update.write_u8(self.update_condition)?;
    update.write_u8(self.update_weapon_flags)?;
    update.write_u16::<T>(self.update_ammo_elapsed)?;
    update.write_u8(self.update_addon_flags)?;
    update.write_u8(self.update_ammo_type)?;
    update.write_u8(self.update_weapon_state)?;
    update.write_u8(self.update_zoom)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemWeapon {
  /// Import ALife weapon item object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectItem::import(section_name, ltx)?,
      ammo_current: section.read_field("item_weapon.ammo_current")?,
      ammo_elapsed: section.read_field("item_weapon.ammo_elapsed")?,
      weapon_state: section.read_field("item_weapon.weapon_state")?,
      addon_flags: section.read_field("item_weapon.addon_flags")?,
      ammo_type: section.read_field("item_weapon.ammo_type")?,
      elapsed_grenades: section.read_field("item_weapon.elapsed_grenades")?,
      update_condition: section.read_field("item_weapon.update_condition")?,
      update_weapon_flags: section.read_field("item_weapon.update_weapon_flags")?,
      update_ammo_elapsed: section.read_field("item_weapon.update_ammo_elapsed")?,
      update_addon_flags: section.read_field("item_weapon.update_addon_flags")?,
      update_ammo_type: section.read_field("item_weapon.update_ammo_type")?,
      update_weapon_state: section.read_field("item_weapon.update_weapon_state")?,
      update_zoom: section.read_field("item_weapon.update_zoom")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item_weapon.ammo_current", self.ammo_current.to_string())
      .set("item_weapon.ammo_elapsed", self.ammo_elapsed.to_string())
      .set("item_weapon.weapon_state", self.weapon_state.to_string())
      .set("item_weapon.addon_flags", self.addon_flags.to_string())
      .set("item_weapon.ammo_type", self.ammo_type.to_string())
      .set("item_weapon.elapsed_grenades", self.elapsed_grenades.to_string())
      .set("item_weapon.update_condition", self.update_condition.to_string())
      .set("item_weapon.update_weapon_flags", self.update_weapon_flags.to_string())
      .set("item_weapon.update_ammo_elapsed", self.update_ammo_elapsed.to_string())
      .set("item_weapon.update_addon_flags", self.update_addon_flags.to_string())
      .set("item_weapon.update_ammo_type", self.update_ammo_type.to_string())
      .set("item_weapon.update_weapon_state", self.update_weapon_state.to_string())
      .set("item_weapon.update_zoom", self.update_zoom.to_string());

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectItemWeapon {
  /// A loaded weapon placed at `position`, its update packet agreeing with its spawn packet.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectItem::new_mock(position),
      ammo_current: 25,
      ammo_elapsed: 5,
      weapon_state: 1,
      addon_flags: 2,
      ammo_type: 1,
      elapsed_grenades: 0,
      update_condition: 214,
      update_weapon_flags: 0,
      update_ammo_elapsed: 5,
      update_addon_flags: 2,
      update_ammo_type: 1,
      update_weapon_state: 1,
      update_zoom: 0,
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
  use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;

  /// Eight bytes follow the item's physics state in the update packet: `CSE_ALifeItemWeapon::UPDATE_Write`.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(original.read_back::<XRayByteOrder>()?, (60, 62, original));

    Ok(())
  }

  #[test]
  fn reads_the_condition_its_update_packet_quantises() {
    let mut weapon: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::ZERO);

    weapon.update_condition = 0;
    assert_eq!(weapon.get_update_condition(), 0.0);

    weapon.update_condition = 255;
    assert!((weapon.get_update_condition() - 1.0).abs() < 1e-6);
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::new(4.0, 5.0, 6.0));
    let mut ltx: Ltx = Ltx::new();

    original.update_zoom = 1;
    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemWeapon::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::new(4.0, 5.0, 6.0));

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemWeapon>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
