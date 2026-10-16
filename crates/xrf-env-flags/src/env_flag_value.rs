/// What a variable's value is read as.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvFlagValue {
  /// On or off.
  Switch,
  /// On or off as wgpu reads its own: `0` off, any other value on.
  ZeroOffSwitch,
  /// One of the names given, in any case.
  Choice(&'static [&'static str]),
  /// A path to a file or a directory.
  Path,
  /// Text its reader parses itself, such as a log filter.
  Text,
}

impl EnvFlagValue {
  /// What a switch takes for on, in any case.
  pub const SWITCH_ON: [&'static str; 4] = ["1", "true", "on", "yes"];
  /// What a switch takes for off, in any case.
  pub const SWITCH_OFF: [&'static str; 4] = ["0", "false", "off", "no"];

  /// The value `raw` stands for, as the variable takes it: a switch's `on` or `off`, a choice's own spelling, a path or
  /// text as set; none where it takes no such value.
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
      Self::ZeroOffSwitch => Some(String::from(if trimmed == "0" { "off" } else { "on" })),
      Self::Choice(names) => names
        .iter()
        .find(|name| name.eq_ignore_ascii_case(trimmed))
        .map(|name| String::from(*name)),
      Self::Path | Self::Text => (!trimmed.is_empty()).then(|| String::from(trimmed)),
    }
  }

  /// What it takes, as a sentence's object: "`0` or `1`, ...".
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
