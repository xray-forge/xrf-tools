use std::fmt::Debug;
use std::ops::RangeInclusive;

use xrf_error::{XrfError, XrfResult};

use crate::data::alife::script_dialect::ScriptDialect;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;

/// Which fields an object's packets carry, from the spawn version and script version its spawn packet header stores.
///
/// The engine gates a class's fields on the spawn version (`m_wVersion`), a script class's own fields on the script
/// version; a field the layout lacks is read as the engine leaves it and written as nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpawnLayout {
  pub(crate) version: u16,
  pub(crate) script_version: u16,
}

impl SpawnLayout {
  /// Shadow of Chernobyl's spawn version (`xrServer_Objects.h:154` of its engine), whose layout is not read.
  pub(crate) const SHADOW_OF_CHERNOBYL_VERSION: u16 = 118;

  /// The spawn versions of Clear Sky's layout its levels hold, 122 to its release's 124 (`xrServer_Objects.h:162` of
  /// its engine). Below 122 an item's update packet and a smart cover's spawn fields take another layout
  /// (`CSE_ALifeInventoryItem::UPDATE_Read`, `CSE_SmartCover::STATE_Read`), which no shipped object carries.
  pub(crate) const CLEAR_SKY_VERSIONS: RangeInclusive<u16> = 122..=124;

  /// Call of Pripyat's spawn version (`xrServer_Objects.h:168`), which the forks built on it keep.
  pub(crate) const CALL_OF_PRIPYAT_VERSION: u16 = 128;

  /// The first spawn version whose smart cover stores whether it can fire (`CSE_SmartCover::STATE_Read`).
  pub(crate) const SMART_COVER_FIRE_FLAG_VERSION: u16 = 128;

  /// The first script version whose `se_smart_cover` stores its last description and loopholes
  /// (`se_smart_cover.script`'s `STATE_Read`).
  pub(crate) const SMART_COVER_LOOPHOLES_SCRIPT_VERSION: u16 = 9;

  /// The first spawn version whose inventory box stores whether it can be taken from, whether it is closed, and its
  /// tip; the engine reads them above 124 (`CSE_ALifeInventoryBox::STATE_Read`).
  pub(crate) const INVENTORY_BOX_ACCESS_VERSION: u16 = 125;

  /// The first spawn version whose climbable object stores its game material; the engine reads it above 126
  /// (`CSE_ALifeObjectClimable::STATE_Read`).
  pub(crate) const CLIMABLE_MATERIAL_VERSION: u16 = 127;

  /// The first spawn version whose inventory item stores its installed upgrades; the engine reads them above 123
  /// (`CSE_ALifeInventoryItem::STATE_Read`).
  pub(crate) const ITEM_UPGRADES_VERSION: u16 = 124;

  /// The first spawn version whose weapon stores its grenade launcher's loaded grenades; the engine reads them above
  /// 122 (`CSE_ALifeItemWeapon::STATE_Read`).
  pub(crate) const WEAPON_GRENADES_VERSION: u16 = 123;

  /// The first spawn version whose trader stores whether its corpse can be taken from and whether it is closed; the
  /// engine reads them above 124 (`CSE_ALifeTraderAbstract::STATE_Read`).
  pub(crate) const TRADER_CORPSE_ACCESS_VERSION: u16 = 125;

  /// The script version Clear Sky's script classes write (`current_server_entity_version` in its `system.ltx`), the
  /// one every Clear Sky object of the reference levels carries. Its scripts read fewer fields below it, a layout no
  /// shipped object holds and which is not read.
  pub(crate) const CLEAR_SKY_SCRIPT_VERSION: u16 = 8;

  /// Whether objects of spawn `version` are read and written, Clear Sky's or Call of Pripyat's.
  pub(crate) fn is_supported_version(version: u16) -> bool {
    Self::CLEAR_SKY_VERSIONS.contains(&version) || version == Self::CALL_OF_PRIPYAT_VERSION
  }

  /// Which game's script classes the object follows, by its spawn version.
  pub(crate) fn script_dialect(self) -> ScriptDialect {
    if Self::CLEAR_SKY_VERSIONS.contains(&self.version) {
      ScriptDialect::ClearSky
    } else {
      ScriptDialect::CallOfPripyat
    }
  }

  /// Refuse a Clear Sky script class's fields at another script version than the one its scripts write.
  ///
  /// # Errors
  ///
  /// Returns an error naming `owner` when the script version is not [`Self::CLEAR_SKY_SCRIPT_VERSION`].
  pub(crate) fn assert_clear_sky_script_version(self, owner: &str) -> XrfResult {
    if self.script_version == Self::CLEAR_SKY_SCRIPT_VERSION {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "{owner} of script version {} is not read: Clear Sky's scripts write version {}",
      self.script_version,
      Self::CLEAR_SKY_SCRIPT_VERSION
    )))
  }

  /// Whether a smart cover stores whether it can fire.
  pub(crate) fn has_smart_cover_fire_flag(self) -> bool {
    self.version >= Self::SMART_COVER_FIRE_FLAG_VERSION
  }

  /// Whether a script smart cover stores its last description and loopholes.
  pub(crate) fn has_smart_cover_loopholes(self) -> bool {
    self.script_version >= Self::SMART_COVER_LOOPHOLES_SCRIPT_VERSION
  }

  /// Whether an inventory box stores its access fields and tip.
  pub(crate) fn has_inventory_box_access(self) -> bool {
    self.version >= Self::INVENTORY_BOX_ACCESS_VERSION
  }

  /// Whether a climbable object stores its game material.
  pub(crate) fn has_climable_material(self) -> bool {
    self.version >= Self::CLIMABLE_MATERIAL_VERSION
  }

  /// Whether an inventory item stores its installed upgrades.
  pub(crate) fn has_item_upgrades(self) -> bool {
    self.version >= Self::ITEM_UPGRADES_VERSION
  }

  /// Whether a weapon stores its loaded grenades.
  pub(crate) fn has_weapon_grenades(self) -> bool {
    self.version >= Self::WEAPON_GRENADES_VERSION
  }

  /// Whether a trader stores its corpse's access fields.
  pub(crate) fn has_trader_corpse_access(self) -> bool {
    self.version >= Self::TRADER_CORPSE_ACCESS_VERSION
  }

  /// Refuse a value of a field the layout lacks unless it is `default`, the value the engine holds in its place, so a
  /// write never drops what the packet cannot carry.
  ///
  /// # Errors
  ///
  /// Returns an error when `value` differs from `default`.
  pub(crate) fn assert_absent_default<V: Debug + PartialEq>(self, field: &str, value: &V, default: &V) -> XrfResult {
    if value == default {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "Field '{field}' holds {value:?}, which spawn version {} and script version {} have no place for: the engine \
       reads {default:?} there",
      self.version, self.script_version
    )))
  }
}

#[cfg(test)]
impl SpawnLayout {
  /// The layout of the packets Call of Pripyat writes, at its spawn version and script version 12.
  pub(crate) const CALL_OF_PRIPYAT: Self = Self {
    version: Self::CALL_OF_PRIPYAT_VERSION,
    script_version: 12,
  };

  /// The layout of Clear Sky's release packets, at its spawn version and script version.
  pub(crate) const CLEAR_SKY: Self = Self {
    version: 124,
    script_version: Self::CLEAR_SKY_SCRIPT_VERSION,
  };
}

impl From<&SpawnPacketHeader> for SpawnLayout {
  fn from(header: &SpawnPacketHeader) -> Self {
    Self {
      version: header.version,
      script_version: header.script_version,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::data::alife::script_dialect::ScriptDialect;
  use crate::data::alife::spawn_layout::SpawnLayout;

  #[test]
  fn supports_clear_sky_and_call_of_pripyat_versions_only() {
    let supported: Vec<u16> = (0..=u16::from(u8::MAX))
      .filter(|version| SpawnLayout::is_supported_version(*version))
      .collect();

    assert_eq!(supported, vec![122, 123, 124, 128]);
  }

  #[test]
  fn gates_each_field_at_its_first_version() {
    let layout = |version: u16, script_version: u16| SpawnLayout {
      version,
      script_version,
    };

    assert!(!layout(124, 12).has_smart_cover_fire_flag());
    assert!(layout(128, 12).has_smart_cover_fire_flag());
    assert!(!layout(128, 8).has_smart_cover_loopholes());
    assert!(layout(128, 9).has_smart_cover_loopholes());
    assert!(!layout(124, 12).has_inventory_box_access());
    assert!(layout(128, 8).has_inventory_box_access());
    assert!(!layout(124, 8).has_climable_material());
    assert!(layout(128, 12).has_climable_material());
    assert!(!layout(123, 8).has_item_upgrades());
    assert!(layout(124, 8).has_item_upgrades());
    assert!(!layout(122, 8).has_weapon_grenades());
    assert!(layout(123, 8).has_weapon_grenades());
    assert!(!layout(124, 8).has_trader_corpse_access());
    assert!(layout(128, 12).has_trader_corpse_access());
  }

  #[test]
  fn picks_the_dialect_by_spawn_version() {
    for version in [122, 123, 124] {
      assert_eq!(
        SpawnLayout {
          version,
          script_version: 8
        }
        .script_dialect(),
        ScriptDialect::ClearSky,
        "{version}"
      );
    }

    // The script version does not decide it: a Call of Pripyat packet at Clear Sky's script version stays Call of
    // Pripyat's.
    assert_eq!(
      SpawnLayout {
        version: 128,
        script_version: 8
      }
      .script_dialect(),
      ScriptDialect::CallOfPripyat
    );
  }

  #[test]
  fn refuses_clear_sky_script_fields_of_another_script_version() {
    assert!(
      SpawnLayout::CLEAR_SKY
        .assert_clear_sky_script_version("se_monster")
        .is_ok()
    );

    for script_version in [7, 9] {
      let layout: SpawnLayout = SpawnLayout {
        version: 123,
        script_version,
      };

      assert_eq!(
        layout
          .assert_clear_sky_script_version("se_monster")
          .expect_err("another script version")
          .to_string(),
        format!(
          "Invalid error: se_monster of script version {script_version} is not read: Clear Sky's scripts write \
           version 8"
        )
      );
    }
  }

  #[test]
  fn refuses_a_value_an_absent_field_cannot_hold() {
    let layout: SpawnLayout = SpawnLayout {
      version: 124,
      script_version: 8,
    };

    assert!(layout.assert_absent_default("can_fire", &1u8, &1u8).is_ok());
    assert_eq!(
      layout
        .assert_absent_default("can_fire", &0u8, &1u8)
        .expect_err("a value other than the engine's default")
        .to_string(),
      "Invalid error: Field 'can_fire' holds 0, which spawn version 124 and script version 8 have no place for: the \
       engine reads 1 there"
    );
  }
}
