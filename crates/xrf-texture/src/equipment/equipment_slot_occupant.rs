use serde::Serialize;
use xrf_ltx::{Ltx, Section};

use crate::equipment::{EquipmentGridRect, EquipmentSlotClaim, LTX_FIELD_INVENTORY_ICON_PATH};

/// A section that occupies part of an equipment sheet, as the editor reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSlotOccupant {
  pub section: String,
  pub claim: EquipmentSlotClaim,
  /// Where the section reads its icon from, when it overrides the default of `<section>.dds` beside the source.
  pub custom_icon: Option<String>,
  /// Engine identity of the config whose header declared the section, where the reader knew one.
  pub origin: Option<String>,
  pub x: u32,
  pub y: u32,
  pub w: u32,
  pub h: u32,
}

impl EquipmentSlotOccupant {
  /// Every section of a resolved config that occupies a slot, in the order the config declares them.
  pub fn new_list_from_ltx(ltx: &Ltx) -> Vec<Self> {
    let mut occupants: Vec<Self> = Vec::new();

    for (section_name, section) in ltx.iter() {
      if let Some(occupant) = Self::new_optional_from_section(section_name, section) {
        occupants.push(occupant);
      }
    }

    occupants
  }

  /// Describes the slot a section occupies, if it occupies one.
  ///
  /// Placement alone qualifies a section, unlike [`crate::InventorySpriteDescriptor`], which needs the
  /// `$inventory_icon` marker because it decides what to write. Placement is enough on every real tree: of the
  /// grid-complete sections in vanilla Call of Pripyat and in Anomaly, not one is an abstract base — every one
  /// resolves a `class` or a `$spawn` somewhere in its chain. Resolved lookups see inherited fields, so an upgrade or
  /// `_nimble` variant reports at its base weapon's slot, which is where the engine draws it.
  pub fn new_optional_from_section<T>(section_name: T, section: &Section) -> Option<Self>
  where
    T: Into<String>,
  {
    let claim: EquipmentSlotClaim = EquipmentSlotClaim::of_section(section)?;
    let rect: EquipmentGridRect = EquipmentGridRect::new_optional_from_section(section)?;

    Some(Self {
      section: section_name.into(),
      claim,
      custom_icon: section.get(LTX_FIELD_INVENTORY_ICON_PATH).map(Into::into),
      origin: None,
      x: rect.x,
      y: rect.y,
      w: rect.w,
      h: rect.h,
    })
  }

  /// Records which config declared this section's header.
  ///
  /// Separate from reading the section because only a resolution knows it; a reader over a flat config has nobody to
  /// ask, and `None` means exactly that rather than a file with no name.
  pub fn with_origin<T>(mut self, origin: Option<T>) -> Self
  where
    T: Into<String>,
  {
    self.origin = origin.map(Into::into);

    self
  }
}
