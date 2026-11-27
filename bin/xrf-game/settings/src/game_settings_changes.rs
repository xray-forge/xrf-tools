use std::collections::BTreeSet;

use crate::game_settings_group::GameSettingsGroup;

/// What changed in the settings since they were last taken, by group: each part of the game takes up the groups it
/// reads, the view its drawing, the window none until it opens again.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameSettingsChanges {
  groups: BTreeSet<GameSettingsGroup>,
}

impl GameSettingsChanges {
  pub(crate) fn insert(&mut self, group: GameSettingsGroup) {
    self.groups.insert(group);
  }

  /// Whether a setting of `group` changed.
  pub fn has(&self, group: GameSettingsGroup) -> bool {
    self.groups.contains(&group)
  }

  /// Whether any setting of `groups` changed.
  pub fn has_any(&self, groups: &[GameSettingsGroup]) -> bool {
    groups.iter().any(|group| self.has(*group))
  }

  pub fn is_empty(&self) -> bool {
    self.groups.is_empty()
  }
}
