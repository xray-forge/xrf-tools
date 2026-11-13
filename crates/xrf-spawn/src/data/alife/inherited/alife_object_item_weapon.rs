use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
#[cfg(any(test, feature = "mocks"))]
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;

/// A weapon, `CSE_ALifeItemWeapon`. Its update packet repeats the rounds loaded, weapon state, addons and ammo type the
/// spawn packet stores, and the engine reads them over the spawn packet's, so those are what the game starts with. Up
/// to spawn version 122 the spawn packet stores no loaded grenades, which then hold the constructor's none of the first
/// type (`xrServer_Objects_ALife_Items.cpp:490-491`, read above 122 at `:565`), the byte `pack_to_byte` makes zero.
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

  /// `elapsed_grenades` where the packet does not store it: no grenades of the first type.
  pub(crate) const DEFAULT_ELAPSED_GRENADES: u8 = 0;

  /// Refuse loaded grenades the layout has no place for unless they are the constructor's none.
  fn assert_grenades_fit(&self, layout: SpawnLayout) -> XrfResult {
    if layout.has_weapon_grenades() {
      return Ok(());
    }

    layout.assert_absent_default(
      "elapsed_grenades",
      &self.elapsed_grenades,
      &Self::DEFAULT_ELAPSED_GRENADES,
    )
  }
}

impl AlifePacketsReadWrite for AlifeObjectItemWeapon {
  /// The item's, then what `CSE_ALifeItemWeapon::STATE_Read` reads (`xrServer_Objects_ALife_Items.cpp:556-563`): the
  /// condition quantised, the rounds loaded, the state, the addons and the ammo type; no weapon flags, as the
  /// constructor leaves them. Whether it aims is uninitialised.
  fn set_spawned_update(&mut self, header: &SpawnPacketHeader) {
    self.base.set_spawned_update(header);
    self.update_condition = AlifeObjectItem::quantize_condition(self.base.condition);
    self.update_weapon_flags = 0;
    self.update_ammo_elapsed = self.ammo_elapsed;
    self.update_addon_flags = self.addon_flags;
    self.update_ammo_type = self.ammo_type;
    self.update_weapon_state = self.weapon_state;
  }

  /// Read a weapon from both packets.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _, _>(spawn, update, layout)?,
      ammo_current: spawn.read_u16::<T>()?,
      ammo_elapsed: spawn.read_u16::<T>()?,
      weapon_state: spawn.read_u8()?,
      addon_flags: spawn.read_u8()?,
      ammo_type: spawn.read_u8()?,
      elapsed_grenades: if layout.has_weapon_grenades() {
        spawn.read_u8()?
      } else {
        Self::DEFAULT_ELAPSED_GRENADES
      },
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
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult {
    self.assert_grenades_fit(layout)?;
    self.base.write::<T, _, _>(spawn, update, layout)?;

    spawn.write_u16::<T>(self.ammo_current)?;
    spawn.write_u16::<T>(self.ammo_elapsed)?;
    spawn.write_u8(self.weapon_state)?;
    spawn.write_u8(self.addon_flags)?;
    spawn.write_u8(self.ammo_type)?;

    if layout.has_weapon_grenades() {
      spawn.write_u8(self.elapsed_grenades)?;
    }

    update.write_u8(self.update_condition)?;
    update.write_u8(self.update_weapon_flags)?;
    update.write_u16::<T>(self.update_ammo_elapsed)?;
    update.write_u8(self.update_addon_flags)?;
    update.write_u8(self.update_ammo_type)?;
    update.write_u8(self.update_weapon_state)?;
    update.write_u8(self.update_zoom)?;

    Ok(())
  }

  /// Import ALife weapon item object data from ltx config section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectItem::import(section_name, ltx, layout)?,
      ammo_current: section.read_field("item_weapon.ammo_current")?,
      ammo_elapsed: section.read_field("item_weapon.ammo_elapsed")?,
      weapon_state: section.read_field("item_weapon.weapon_state")?,
      addon_flags: section.read_field("item_weapon.addon_flags")?,
      ammo_type: section.read_field("item_weapon.ammo_type")?,
      elapsed_grenades: if layout.has_weapon_grenades() {
        section.read_field("item_weapon.elapsed_grenades")?
      } else {
        Self::DEFAULT_ELAPSED_GRENADES
      },
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
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.assert_grenades_fit(layout)?;
    self.base.export(section_name, ltx, layout)?;

    ltx
      .with_section(section_name)
      .set("item_weapon.ammo_current", self.ammo_current)
      .set("item_weapon.ammo_elapsed", self.ammo_elapsed)
      .set("item_weapon.weapon_state", self.weapon_state)
      .set("item_weapon.addon_flags", self.addon_flags)
      .set("item_weapon.ammo_type", self.ammo_type);

    if layout.has_weapon_grenades() {
      ltx
        .with_section(section_name)
        .set("item_weapon.elapsed_grenades", self.elapsed_grenades);
    }

    ltx
      .with_section(section_name)
      .set("item_weapon.update_condition", self.update_condition)
      .set("item_weapon.update_weapon_flags", self.update_weapon_flags)
      .set("item_weapon.update_ammo_elapsed", self.update_ammo_elapsed)
      .set("item_weapon.update_addon_flags", self.update_addon_flags)
      .set("item_weapon.update_ammo_type", self.update_ammo_type)
      .set("item_weapon.update_weapon_state", self.update_weapon_state)
      .set("item_weapon.update_zoom", self.update_zoom);

    Ok(())
  }
}

#[cfg(any(test, feature = "mocks"))]
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
  use xrf_chunk::{ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;
  use crate::data::alife::spawn_layout::SpawnLayout;

  fn layout(version: u16) -> SpawnLayout {
    SpawnLayout {
      version,
      script_version: 8,
    }
  }

  /// The grenade byte is the weapon's one spawn field the layout gates; at 123 the item stores no upgrade count.
  #[test]
  fn writes_grenades_above_version_122_only() -> XrfResult {
    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let (with_grenades, update_with, read_with) = original.read_back::<XRayByteOrder>(layout(123))?;
    let (without_grenades, update_without, read_without) = original.read_back::<XRayByteOrder>(layout(122))?;

    assert_eq!(with_grenades, 60 - 4);
    assert_eq!(without_grenades, with_grenades - 1);
    assert_eq!(update_without, update_with);
    assert_eq!(read_with, original);
    assert_eq!(read_without, original);

    Ok(())
  }

  #[test]
  fn refuses_to_drop_grenades_up_to_version_122() {
    let mut original: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::ZERO);

    original.elapsed_grenades = 3;

    let error: String = original
      .write::<XRayByteOrder, _, _>(&mut ChunkWriter::new(), &mut ChunkWriter::new(), layout(122))
      .expect_err("grenades the packet cannot hold")
      .to_string();

    assert!(
      error.starts_with("Invalid error: Field 'elapsed_grenades' holds"),
      "{error}"
    );
    assert!(original.export("data", &mut Ltx::new(), layout(122)).is_err());
  }

  #[test]
  fn imports_and_exports_no_grenades_up_to_version_122() -> XrfResult {
    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::new(4.0, 5.0, 6.0));
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx, layout(122))?;

    assert_eq!(ltx.get_from("data", "item_weapon.elapsed_grenades"), None);
    assert_eq!(AlifeObjectItemWeapon::import("data", &ltx, layout(122))?, original);

    Ok(())
  }

  /// Eight bytes follow the item's physics state in the update packet: `CSE_ALifeItemWeapon::UPDATE_Write`.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (60, 62, original)
    );

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
    original.export("data", &mut ltx, SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(
      AlifeObjectItemWeapon::import("data", &ltx, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );

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
