use crate::project::info_portions::info_portion_place::InfoPortionPlace;

/// How a tree uses one info portion: the first place each kind of use is written, and what scripts, condlist functions
/// and its declaration do with it beyond them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct InfoPortionUses {
  /// The first `info_*.xml` declaring it.
  pub(crate) declared: Option<InfoPortionPlace>,
  /// The first condlist effect, dialog `give_info`, known info section, spawned document or script giver call giving
  /// it.
  pub(crate) given: Option<InfoPortionPlace>,
  /// The first `give_info` of a dialog file `system.ltx` does not list, which the engine never loads.
  pub(crate) given_unlisted: Option<InfoPortionPlace>,
  /// The first debug scene starting with it, which a developer menu gives to jump into its quest.
  pub(crate) given_by_debug_scene: Option<InfoPortionPlace>,
  /// The first condlist effect, dialog `disable_info` or another portion's `<disable>` taking it away.
  pub(crate) taken: Option<InfoPortionPlace>,
  /// The first condlist condition or dialog `has_info` or `dont_has_info` the engine reads it in.
  pub(crate) read: Option<InfoPortionPlace>,
  /// Whether a script reads it through a reader call, such as `has_alife_info`.
  pub(crate) is_script_read: bool,
  /// Whether a script names it where what it does is unknown: in a call this table does not know, a table kept as
  /// data, or a name built onto a literal prefix, any of which may give or read it.
  pub(crate) is_scripted: bool,
  /// Whether a condlist function receives it as a parameter, which may give or read it.
  pub(crate) is_passed: bool,
  /// Whether holding it does something by itself: its declaration opens dialogs or takes other portions away.
  pub(crate) is_effective: bool,
  /// Whether the engine gives it itself, as a window opens or closes, so that what reads it is answered whatever the
  /// tree writes.
  pub(crate) is_given_by_engine: bool,
}

impl InfoPortionUses {
  /// Where something first reads the portion, when nothing gives it: no condlist, dialog the engine loads, known info
  /// section, spawned document or script giver call, no script naming it otherwise, no condlist function receiving it,
  /// no debug scene starting with it, which is the author's own entry into the quest, and not the engine.
  pub(crate) fn find_read_only(&self) -> Option<&InfoPortionPlace> {
    self.read.as_ref().filter(|_| !self.is_given_anyhow())
  }

  /// Whether anything may give the portion: a giver the tree writes, a debug scene, the engine itself, or a script or
  /// condlist function naming it with a role unknown.
  fn is_given_anyhow(&self) -> bool {
    self.given.is_some()
      || self.given_by_debug_scene.is_some()
      || self.is_given_by_engine
      || self.is_scripted
      || self.is_passed
  }

  /// Where something first gives the portion, when nothing reads it and holding it does nothing: no condlist or dialog
  /// reads it, no script reads or otherwise names it, no condlist function receives it, and its declaration opens no
  /// dialog and takes nothing away.
  pub(crate) fn find_unread(&self) -> Option<&InfoPortionPlace> {
    self.given.as_ref().filter(|_| {
      self.read.is_none() && !self.is_script_read && !self.is_scripted && !self.is_passed && !self.is_effective
    })
  }

  /// Where the portion is declared, when its declaration does something that can never happen: it opens a dialog or
  /// takes other portions away, and nothing gives it, as `find_read_only` judges giving.
  pub(crate) fn find_unreachable(&self) -> Option<&InfoPortionPlace> {
    self
      .declared
      .as_ref()
      .filter(|_| self.is_effective && !self.is_given_anyhow())
  }
}
