/// One setting as the console and the settings file write it: its dotted path in snake case, as
/// `view.shadows.resolution`, and its value as text, as `2048`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameSettingEntry {
  pub path: String,
  pub value: String,
}
