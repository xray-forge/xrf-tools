use crate::project::info_portions::info_portion_place::InfoPortionPlace;

/// How a tree uses one info portion: the first place each kind of use is written, and whether a script or a condlist
/// function may give it out of sight.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct InfoPortionUses {
  /// The first `info_*.xml` declaring it.
  pub(crate) declared: Option<InfoPortionPlace>,
  /// The first condlist effect, dialog `give_info` or spawned document giving it.
  pub(crate) given: Option<InfoPortionPlace>,
  /// The first `give_info` of a dialog file `system.ltx` does not list, which the engine never loads.
  pub(crate) given_unlisted: Option<InfoPortionPlace>,
  /// The first debug scene starting with it, which a developer menu gives to jump into its quest.
  pub(crate) given_by_debug_scene: Option<InfoPortionPlace>,
  /// The first condlist effect or dialog `disable_info` taking it away.
  pub(crate) taken: Option<InfoPortionPlace>,
  /// The first condlist condition or dialog `has_info` or `dont_has_info` the engine reads it in.
  pub(crate) read: Option<InfoPortionPlace>,
  /// Whether a script passes its name straight to a call, keeps it in a table as data or builds it onto a literal
  /// prefix, any of which may give it.
  pub(crate) is_scripted: bool,
  /// Whether a condlist function receives it as a parameter, which may give it.
  pub(crate) is_passed: bool,
}

impl InfoPortionUses {
  /// Where something first reads the portion, when nothing gives it: no condlist, dialog the engine loads or spawned
  /// document, no script naming it, no condlist function receiving it, and no debug scene starting with it, which is
  /// the author's own entry into the quest.
  pub(crate) fn find_read_only(&self) -> Option<&InfoPortionPlace> {
    self
      .read
      .as_ref()
      .filter(|_| self.given.is_none() && self.given_by_debug_scene.is_none() && !self.is_scripted && !self.is_passed)
  }
}
