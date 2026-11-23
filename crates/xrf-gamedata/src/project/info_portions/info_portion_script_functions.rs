use crate::project::info_portions::info_portion_role::InfoPortionRole;

/// What a script call does with the info portion names passed to it, by the function it calls.
///
/// Vanilla gives through the actor's `give_info_portion` and `xr_effects.give_info`, takes through
/// `disable_info_portion` and `xr_effects.disable_info`, and reads through `has_alife_info`, `dont_has_alife_info` and
/// the object's `has_info` and `dont_has_info`. xrf-engine's compiled scripts call `giveInfoPortion`,
/// `disableInfoPortion` and `hasInfoPortion` with the name inlined (`core/utils/info_portion.ts`); its other readers take
/// arrays, whose names a table holds. Any other call does something unknown with the names it is passed.
pub(crate) struct InfoPortionScriptFunctions;

impl InfoPortionScriptFunctions {
  const GIVERS: [&'static str; 3] = ["give_info_portion", "give_info", "giveInfoPortion"];
  const TAKERS: [&'static str; 3] = ["disable_info_portion", "disable_info", "disableInfoPortion"];
  const READERS: [&'static str; 5] = [
    "has_alife_info",
    "dont_has_alife_info",
    "has_info",
    "dont_has_info",
    "hasInfoPortion",
  ];

  /// The role a call of a function gives the names passed to it, or `None` for a function this table does not know.
  pub(crate) fn find_role(function: &str) -> Option<InfoPortionRole> {
    if Self::GIVERS.contains(&function) {
      Some(InfoPortionRole::Given)
    } else if Self::TAKERS.contains(&function) {
      Some(InfoPortionRole::Taken)
    } else if Self::READERS.contains(&function) {
      Some(InfoPortionRole::Read)
    } else {
      None
    }
  }
}
