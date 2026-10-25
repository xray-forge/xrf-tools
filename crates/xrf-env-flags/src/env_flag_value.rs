/// How a variable's value is parsed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvFlagValue {
  /// On or off.
  Switch,
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
    match self {
      Self::Switch => Self::parse_switch(raw).map(|is_on| String::from(if is_on { "on" } else { "off" })),
      Self::Choice(names) => Self::parse_choice(names, raw).map(String::from),
      Self::Path | Self::Text => Self::parse_text(raw),
    }
  }

  /// The accepted values, phrased to follow "Takes:", for example "`d3d12` or `vulkan`".
  pub fn describe(self) -> String {
    match self {
      Self::Switch => format!(
        "{} for on; {} for off",
        Self::describe_names(&Self::SWITCH_ON),
        Self::describe_names(&Self::SWITCH_OFF)
      ),
      Self::Choice(names) => Self::describe_names(names),
      Self::Path => String::from("a path"),
      Self::Text => String::from("text"),
    }
  }

  /// A switch's state, whatever its padding or case.
  pub(crate) fn parse_switch(raw: &str) -> Option<bool> {
    let lower: String = raw.trim().to_ascii_lowercase();

    if Self::SWITCH_ON.contains(&lower.as_str()) {
      Some(true)
    } else if Self::SWITCH_OFF.contains(&lower.as_str()) {
      Some(false)
    } else {
      None
    }
  }

  /// The name `raw` chooses, as `names` spells it, whatever its padding or case.
  pub(crate) fn parse_choice(names: &'static [&'static str], raw: &str) -> Option<&'static str> {
    let trimmed: &str = raw.trim();

    names.iter().copied().find(|name| name.eq_ignore_ascii_case(trimmed))
  }

  /// The trimmed text, or none if nothing is left.
  pub(crate) fn parse_text(raw: &str) -> Option<String> {
    let trimmed: &str = raw.trim();

    (!trimmed.is_empty()).then(|| String::from(trimmed))
  }

  /// Names as alternatives: "`a`", "`a` or `b`", "`a`, `b` or `c`".
  fn describe_names(names: &[&str]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();

    match quoted.split_last() {
      Some((last, [])) => last.clone(),
      Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
      None => String::new(),
    }
  }
}
