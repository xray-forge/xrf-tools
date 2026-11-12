use std::fmt::Debug;
use std::ops::RangeInclusive;

use xrf_error::{XrfError, XrfResult};

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

  /// The spawn versions of Clear Sky's layout, up to its release's 124 (`xrServer_Objects.h:162` of its engine); its
  /// levels hold objects of 122 to 124. Version 121 is the first whose spawn packet header carries the game type.
  pub(crate) const CLEAR_SKY_VERSIONS: RangeInclusive<u16> = 121..=124;

  /// Call of Pripyat's spawn version (`xrServer_Objects.h:168`), which the forks built on it keep.
  pub(crate) const CALL_OF_PRIPYAT_VERSION: u16 = 128;

  /// The first spawn version whose smart cover stores its combat cover flag; below it the flag comes from the
  /// section's config (`CSE_SmartCover::STATE_Read`).
  pub(crate) const SMART_COVER_COMBAT_FLAG_VERSION: u16 = 122;

  /// The first spawn version whose smart cover stores whether it can fire (`CSE_SmartCover::STATE_Read`).
  pub(crate) const SMART_COVER_FIRE_FLAG_VERSION: u16 = 128;

  /// The first script version whose `se_smart_cover` stores its last description and loopholes
  /// (`se_smart_cover.script`'s `STATE_Read`).
  pub(crate) const SMART_COVER_LOOPHOLES_SCRIPT_VERSION: u16 = 9;

  /// The first spawn version whose inventory box stores whether it can be taken from, whether it is closed, and its
  /// tip; the engine reads them above 124 (`CSE_ALifeInventoryBox::STATE_Read`).
  pub(crate) const INVENTORY_BOX_ACCESS_VERSION: u16 = 125;

  /// Whether objects of spawn `version` are read and written, Clear Sky's or Call of Pripyat's.
  pub(crate) fn is_supported_version(version: u16) -> bool {
    Self::CLEAR_SKY_VERSIONS.contains(&version) || version == Self::CALL_OF_PRIPYAT_VERSION
  }

  /// Whether a smart cover stores its combat cover flag.
  pub(crate) fn has_smart_cover_combat_flag(self) -> bool {
    self.version >= Self::SMART_COVER_COMBAT_FLAG_VERSION
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
  use crate::data::alife::spawn_layout::SpawnLayout;

  #[test]
  fn supports_clear_sky_and_call_of_pripyat_versions_only() {
    let supported: Vec<u16> = (0..=u16::from(u8::MAX))
      .filter(|version| SpawnLayout::is_supported_version(*version))
      .collect();

    assert_eq!(supported, vec![121, 122, 123, 124, 128]);
  }

  #[test]
  fn gates_each_field_at_its_first_version() {
    let layout = |version: u16, script_version: u16| SpawnLayout {
      version,
      script_version,
    };

    assert!(!layout(121, 8).has_smart_cover_combat_flag());
    assert!(layout(122, 8).has_smart_cover_combat_flag());
    assert!(!layout(124, 12).has_smart_cover_fire_flag());
    assert!(layout(128, 12).has_smart_cover_fire_flag());
    assert!(!layout(128, 8).has_smart_cover_loopholes());
    assert!(layout(128, 9).has_smart_cover_loopholes());
    assert!(!layout(124, 12).has_inventory_box_access());
    assert!(layout(128, 8).has_inventory_box_access());
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
