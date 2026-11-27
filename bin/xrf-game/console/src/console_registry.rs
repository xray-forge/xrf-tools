use std::collections::BTreeMap;

use crate::console_answer::ConsoleAnswer;
use crate::console_command::ConsoleCommand;
use crate::console_context::ConsoleContext;
use crate::console_line::ConsoleLine;
use crate::console_variable::ConsoleVariable;

/// The console's commands over a context `C` that owns its variables, run by the line typed, as X-Ray's console runs
/// them: a command's name and what follows it, a variable's name alone to show it, or with a value to set it. `help`
/// and `list` are its own.
pub struct ConsoleRegistry<C> {
  commands: BTreeMap<&'static str, ConsoleCommand<C>>,
}

impl<C> Default for ConsoleRegistry<C> {
  fn default() -> Self {
    Self {
      commands: BTreeMap::new(),
    }
  }
}

impl<C: ConsoleContext> ConsoleRegistry<C> {
  /// Lists the commands and how variables are used.
  pub const HELP: &'static str = "help";

  /// Lists the variables, those whose name holds what follows it where something does.
  pub const LIST: &'static str = "list";

  /// How many variables an unknown name suggests.
  const SUGGESTED: usize = 5;

  /// The registry with `command` added, replacing one of its name.
  #[must_use]
  pub fn with(mut self, command: ConsoleCommand<C>) -> Self {
    self.commands.insert(command.name, command);

    self
  }

  /// Runs the line `text` against `context`. A blank line answers nothing; a name that is neither a command nor a
  /// variable fails, suggesting variables whose name holds it.
  pub fn execute(&self, context: &mut C, text: &str) -> ConsoleAnswer {
    let Some(line) = ConsoleLine::parse(text) else {
      return ConsoleAnswer::default();
    };

    if line.name == Self::HELP {
      return ConsoleAnswer::succeed(self.describe());
    }

    if line.name == Self::LIST {
      return Self::list(context, line.argument.unwrap_or_default());
    }

    if let Some(command) = self.commands.get(line.name) {
      return (command.run)(context, line.argument);
    }

    let variables: Vec<ConsoleVariable> = context.list_variables();

    if let Some(variable) = variables.iter().find(|variable| variable.name == line.name) {
      return match line.argument {
        Some(value) => context.set_variable(line.name, value),
        None => ConsoleAnswer::succeed(vec![Self::format_variable(variable)]),
      };
    }

    let suggested: Vec<&str> = variables
      .iter()
      .filter(|variable| variable.name.contains(line.name))
      .take(Self::SUGGESTED)
      .map(|variable| variable.name.as_str())
      .collect();

    ConsoleAnswer::fail(if suggested.is_empty() {
      format!("Unknown command or variable '{}'; `help` lists them", line.name)
    } else {
      format!(
        "Unknown command or variable '{}'; did you mean {}?",
        line.name,
        suggested.join(", ")
      )
    })
  }

  /// `help`'s lines: every command with how it is written and what it does, then how variables are used.
  fn describe(&self) -> Vec<String> {
    let mut lines: Vec<String> = vec![
      format!("{} - this list", Self::HELP),
      format!("{} [part] - the variables, those whose name holds part", Self::LIST),
    ];

    lines.extend(self.commands.values().map(|command| {
      if command.usage.is_empty() {
        format!("{} - {}", command.name, command.help)
      } else {
        format!("{} {} - {}", command.name, command.usage, command.help)
      }
    }));
    lines.push(String::from("<variable> shows it, <variable> <value> sets it"));

    lines
  }

  /// `list`'s lines: the variables whose name holds `part`, every one for none.
  fn list(context: &C, part: &str) -> ConsoleAnswer {
    let lines: Vec<String> = context
      .list_variables()
      .iter()
      .filter(|variable| variable.name.contains(part))
      .map(Self::format_variable)
      .collect();

    if lines.is_empty() {
      return ConsoleAnswer::fail(format!("No variable's name holds '{part}'"));
    }

    ConsoleAnswer::succeed(lines)
  }

  fn format_variable(variable: &ConsoleVariable) -> String {
    format!("{} = {}", variable.name, variable.value)
  }
}
