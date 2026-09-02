use clap::{Arg, ArgAction, Command};

use crate::core::generic_command::CommandGroup;
use crate::core::reporting::ReportingArguments;

/// Documentation model extracted from one registered CLI command group.
pub struct GroupReference {
  pub slug: &'static str,
  pub label: &'static str,
  pub commands: Vec<CommandReference>,
}

pub struct CommandReference {
  pub name: String,
  pub about: Option<String>,
  pub usage: String,
  pub arguments: Vec<ArgumentReference>,
}

pub struct ArgumentReference {
  pub display: String,
  pub help: Option<String>,
  pub is_required: bool,
  pub default_value: Option<String>,
  pub possible_values: Vec<String>,
  pub value_delimiter: Option<char>,
}

impl GroupReference {
  pub fn from_group(group: &CommandGroup) -> Self {
    Self {
      slug: group.slug,
      label: group.label,
      commands: group
        .commands
        .iter()
        // A command is documented as a caller meets it, which includes the reporting flags every command answers to.
        .map(|command| CommandReference::from_command(group.slug, command.init().with_reporting()))
        .collect(),
    }
  }
}

impl CommandReference {
  /// Preserves argument declaration order in generated reference pages.
  pub fn from_command(domain: &str, mut command: Command) -> Self {
    command.build();

    let usage: String = Self::render_usage(domain, &mut command);

    Self {
      name: command.get_name().to_string(),
      about: command.get_about().map(|about| about.to_string()),
      usage,
      arguments: command
        .get_arguments()
        .filter(|argument| !argument.is_hide_set() && !matches!(argument.get_id().as_str(), "help" | "version"))
        .map(ArgumentReference::from_argument)
        .collect(),
    }
  }

  /// Restores the binary prefix that registered subcommands inherit from clap.
  fn render_usage(domain: &str, command: &mut Command) -> String {
    let usage: String = command.render_usage().to_string();
    let usage: &str = usage.strip_prefix("Usage: ").unwrap_or(&usage);

    format!("xrf-cli {domain} {usage}")
  }
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

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use clap::{Arg, ArgAction, Command, value_parser};

  use super::CommandReference;

  fn build_reference() -> CommandReference {
    CommandReference::from_command(
      "example",
      Command::new("example-command")
        .about("Example command for docs extraction")
        .arg(
          Arg::new("root")
            .help("Positional path")
            .required(true)
            .value_name("ROOT")
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
          Arg::new("language")
            .help("Target language")
            .short('l')
            .long("language")
            .default_value("all")
            .value_parser(["all", "eng", "ukr"]),
        )
        .arg(
          Arg::new("checks")
            .help("List of checks")
            .long("checks")
            .value_delimiter(',')
            .num_args(1..)
            .value_parser(value_parser!(String)),
        )
        .arg(
          Arg::new("silent")
            .help("Disable any logging")
            .short('s')
            .long("silent")
            .action(ArgAction::SetTrue),
        )
        .arg(
          Arg::new("sort")
            .help("Preserve source order")
            .long("no-sort")
            .action(ArgAction::SetFalse),
        ),
    )
  }

  #[test]
  fn extracts_command_metadata() {
    let reference: CommandReference = build_reference();

    assert_eq!(reference.name, "example-command");
    assert_eq!(reference.about.as_deref(), Some("Example command for docs extraction"));
    assert!(reference.usage.starts_with("xrf-cli example example-command"));
    assert_eq!(reference.arguments.len(), 5);
  }

  #[test]
  fn extracts_argument_shapes() {
    let reference: CommandReference = build_reference();

    let root = &reference.arguments[0];
    assert_eq!(root.display, "<ROOT>");
    assert!(root.is_required);
    assert_eq!(root.default_value, None);

    let language = &reference.arguments[1];
    assert_eq!(language.display, "-l, --language <language>");
    assert!(!language.is_required);
    assert_eq!(language.default_value.as_deref(), Some("all"));
    assert_eq!(language.possible_values, vec!["all", "eng", "ukr"]);

    let checks = &reference.arguments[2];
    assert_eq!(checks.display, "--checks <checks>...");
    assert_eq!(checks.value_delimiter, Some(','));

    let silent = &reference.arguments[3];
    assert_eq!(silent.display, "-s, --silent");
    assert_eq!(silent.default_value, None);

    let sort = &reference.arguments[4];
    assert_eq!(sort.display, "--no-sort");
  }
}
