/// One setting changed over the preset: its dotted path from the settings' root, as `view.shadows.resolution`, its value
/// as written, and where it was written, for the log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameSettingOverride {
  pub path: String,
  pub value: String,
  pub origin: String,
}

impl GameSettingOverride {
  /// The override an argument spells, `<path>=<value>`, none where it has no `=` or no path.
  pub(crate) fn parse_argument(argument: &str) -> Option<Self> {
    let (path, value) = argument.split_once('=')?;
    let path: &str = path.trim();

    (!path.is_empty()).then(|| Self {
      path: path.to_owned(),
      value: value.trim().to_owned(),
      origin: String::from("--set"),
    })
  }
}
