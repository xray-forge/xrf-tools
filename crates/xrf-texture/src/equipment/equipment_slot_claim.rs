use serde::Serialize;
use xrf_ltx::Section;

use crate::equipment::LTX_FIELD_INVENTORY_ICON;

/// How strongly a section claims the slot it names.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EquipmentSlotClaim {
  /// The section resolves `$inventory_icon = true`, so the packing tools act on it.
  Declared,
  /// The section positions itself on the grid but declares no icon of its own.
  Probable,
}

impl EquipmentSlotClaim {
  /// How a section claims its icon, or `None` when it has opted out of having one.
  ///
  /// One reading of `$inventory_icon` for both readers of a sheet, so the packing rule and the editor's are visibly
  /// the same question asked at two thresholds rather than two parses that can answer differently. An explicit
  /// `false` is a refusal, not a weaker claim: a section that has opted out of the packing tools has said something
  /// about its icon, and reporting it anyway would turn that statement into its opposite.
  pub(crate) fn of_section(section: &Section) -> Option<Self> {
    match section.get(LTX_FIELD_INVENTORY_ICON).map(|value| value.parse::<bool>()) {
      Some(Ok(false)) => None,
      Some(Ok(true)) => Some(Self::Declared),
      // An unparseable marker declares nothing, which is what a section without one also does.
      Some(Err(_)) | None => Some(Self::Probable),
    }
  }
}
