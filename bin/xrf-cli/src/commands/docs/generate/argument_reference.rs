use clap::{Arg, ArgAction};

/// Documentation model of one argument, as its reference table row shows it.
pub struct ArgumentReference {
  pub display: String,
  pub help: Option<String>,
  pub is_required: bool,
  pub default_value: Option<String>,
  pub possible_values: Vec<String>,
  pub value_delimiter: Option<char>,
}

impl ArgumentReference {
  pub fn from_argument(argument: &Arg) -> Self {
    let takes_value: bool = matches!(argument.get_action(), ArgAction::Set | ArgAction::Append);

    let value_name: String = argument
      .get_value_names()
      .and_then(|names| names.first().map(|name| name.to_string()))
      .unwrap_or_else(|| argument.get_id().to_string());

    let is_multiple: bool = takes_value
      && (matches!(argument.get_action(), ArgAction::Append)
        || argument.get_num_args().is_some_and(|range| range.max_values() > 1));

    let display: String = if argument.is_positional() {
      format!("<{value_name}>")
    } else {
      let mut flags: Vec<String> = Vec::new();

      if let Some(short) = argument.get_short() {
        flags.push(format!("-{short}"));
      }

      if let Some(long) = argument.get_long() {
        flags.push(format!("--{long}"));
      }

      match (takes_value, is_multiple) {
        (true, true) => format!("{} <{value_name}>...", flags.join(", ")),
        (true, false) => format!("{} <{value_name}>", flags.join(", ")),
        (false, _) => flags.join(", "),
      }
    };

    Self {
      display,
      help: argument.get_help().map(|help| help.to_string()),
      is_required: argument.is_required_set(),
      // Flag actions carry implicit "true"/"false" defaults that would only add table noise.
      default_value: takes_value
        .then(|| {
          argument
            .get_default_values()
            .iter()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<String>>()
        })
        .filter(|values| !values.is_empty())
        .map(|values| values.join(",")),
      possible_values: argument
        .get_possible_values()
        .iter()
        .filter(|value| !value.is_hide_set())
        .map(|value| value.get_name().to_string())
        .collect(),
      value_delimiter: argument.get_value_delimiter(),
    }
  }
}
