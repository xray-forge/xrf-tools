/// How a variable's value is parsed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvFlagValue {
  /// On or off.
  Switch,
  /// On or off as wgpu reads its switches: `0` is off, any other value is on.
  ZeroOffSwitch,
  /// One of the names given, in any case.
  Choice(&'static [&'static str]),
  /// A path to a file or a directory.
  Path,
  /// Text its reader parses itself, such as a log filter.
  Text,
}

impl EnvFlagValue {
  /// Values a switch accepts for on, in any case.
  pub const SWITCH_ON: [&'static str; 4] = ["1", "true", "on", "yes"];
  /// Values a switch accepts for off, in any case.
  pub const SWITCH_OFF: [&'static str; 4] = ["0", "false", "off", "no"];

  /// Parses `raw`: `on` or `off` for a switch, the table's spelling for a choice, the trimmed text for a path or text.
  /// None if the value is invalid.
  pub fn interpret(self, raw: &str) -> Option<String> {
    let trimmed: &str = raw.trim();

    match self {
      Self::Switch => {
        let lower: String = trimmed.to_ascii_lowercase();

        if Self::SWITCH_ON.contains(&lower.as_str()) {
          Some(String::from("on"))
        } else if Self::SWITCH_OFF.contains(&lower.as_str()) {
          Some(String::from("off"))
        } else {
          None
        }
      }
      // Untrimmed, as wgpu compares it: a padded ` 0` turns the switch on.
      Self::ZeroOffSwitch => Some(String::from(if raw == "0" { "off" } else { "on" })),
      Self::Choice(names) => names
        .iter()
        .find(|name| name.eq_ignore_ascii_case(trimmed))
        .map(|name| String::from(*name)),
      Self::Path | Self::Text => (!trimmed.is_empty()).then(|| String::from(trimmed)),
    }
  }

  /// The accepted values, phrased to follow "Takes:", for example "`d3d12` or `vulkan`".
  pub fn describe(self) -> String {
    match self {
      Self::Switch => String::from("`1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off"),
      Self::ZeroOffSwitch => String::from("`0` for off, any other value for on (`false` too)"),
      Self::Choice(names) => {
        let quoted: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();

        match quoted.split_last() {
          Some((last, [])) => last.clone(),
          Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
          None => String::new(),
        }
      }
      Self::Path => String::from("a path"),
      Self::Text => String::from("text"),
    }
  }
}
