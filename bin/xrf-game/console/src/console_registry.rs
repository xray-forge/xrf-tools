use std::collections::BTreeMap;

use crate::console_answer::ConsoleAnswer;
use crate::console_choice::ConsoleChoice;
use crate::console_command::ConsoleCommand;
use crate::console_completion::ConsoleCompletion;
use crate::console_context::ConsoleContext;
use crate::console_line::ConsoleLine;
use crate::console_variable::ConsoleVariable;

/// The console's commands over a context `C` that owns its variables, run by the line typed, as X-Ray's console runs
/// them: a command's name and what follows it, a variable's name alone to show it, or with a value to set it. `help`
/// and `list` are its own.
///
/// `register` is its hook point: any system, and later a script, adds a command there, and one registered under a name
/// already taken, `help` and `list` included, overrides it.
pub struct ConsoleRegistry<C> {
  commands: BTreeMap<String, ConsoleCommand<C>>,
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

  /// Adds `command`, overriding one registered under its name before, answering it.
  pub fn register(&mut self, command: ConsoleCommand<C>) -> Option<ConsoleCommand<C>> {
    self.commands.insert(command.name.clone(), command)
  }

  /// The registry with `command` added, as [`Self::register`] adds it.
  #[must_use]
  pub fn with(mut self, command: ConsoleCommand<C>) -> Self {
    self.register(command);

    self
  }

  /// Runs the line `text` against `context`: a registered command first, then `help` and `list`, then a variable. A
  /// blank line answers nothing; a name that is none of them fails, suggesting variables whose name holds it.
  pub fn execute(&self, context: &mut C, text: &str) -> ConsoleAnswer {
    let Some(line) = ConsoleLine::parse(text) else {
      return ConsoleAnswer::default();
    };

    if let Some(command) = self.commands.get(line.name) {
      return (command.run)(context, line.argument);
    }

    if line.name == Self::HELP {
      return ConsoleAnswer::succeed(self.describe());
    }

    if line.name == Self::LIST {
      return Self::list(context, line.argument.unwrap_or_default());
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

  /// Completes the name `line` starts, as Tab does: to the longest start every command and variable it may still be
  /// share, then a space once one is left; and every choice left, each to its next dot and hinted, as a suggestion
  /// list shows them. A line already past its name, or one no name starts, is left as it is with no choice.
  pub fn complete(&self, context: &C, line: &str) -> ConsoleCompletion {
    let typed: &str = line.trim_start();
    let unchanged = || ConsoleCompletion {
      line: line.to_owned(),
      choices: Vec::new(),
    };

    if typed.is_empty() || typed.contains(char::is_whitespace) {
      return unchanged();
    }

    let variables: Vec<ConsoleVariable> = context.list_variables();
    let mut names: Vec<&str> = self
      .list_builtins()
      .chain(self.commands.keys().map(String::as_str))
      .chain(variables.iter().map(|variable| variable.name.as_str()))
      .filter(|name| name.starts_with(typed))
      .collect();

    names.sort_unstable();
    let Some(first) = names.first() else {
      return unchanged();
    };
    let mut shared: usize = names.iter().fold(first.len(), |shared, name| {
      first
        .bytes()
        .zip(name.bytes())
        .take(shared)
        .take_while(|(left, right)| left == right)
        .count()
    });

    // The shared start is counted in bytes; a name of more than ASCII is cut on a whole character.
    while !first.is_char_boundary(shared) {
      shared -= 1;
    }

    // Each name to its next dot past what they share: the group it opens, or the name itself.
    let mut grouped: Vec<&str> = names
      .iter()
      .map(|name| match name[shared..].find('.') {
        Some(dot) => &name[..=shared + dot],
        None => name,
      })
      .collect();

    grouped.sort_unstable();
    grouped.dedup();

    let choices: Vec<ConsoleChoice> = grouped
      .iter()
      .map(|name| ConsoleChoice {
        name: String::from(*name),
        hint: self.describe_choice(name, &names, &variables),
      })
      .collect();
    let line: String = if names.len() == 1 {
      format!("{first} ")
    } else {
      String::from(&first[..shared])
    };

    ConsoleCompletion { line, choices }
  }

  /// The hint beside the choice `name`: how many of `names` a group holds, a variable's value, or a command's use.
  fn describe_choice(&self, name: &str, names: &[&str], variables: &[ConsoleVariable]) -> String {
    if name.ends_with('.') {
      let count: usize = names.iter().filter(|other| other.starts_with(name)).count();

      return format!("{count} {}", if count == 1 { "variable" } else { "variables" });
    }

    if let Some(variable) = variables.iter().find(|variable| variable.name == name) {
      return format!("= {}", variable.value);
    }

    match self.commands.get(name) {
      Some(command) if command.usage.is_empty() => command.help.clone(),
      Some(command) => format!("{} - {}", command.usage, command.help),
      None if name == Self::HELP => String::from("the commands"),
      None => String::from("[part] - the variables"),
    }
  }

  /// `help` and `list`, those no registered command overrides.
  fn list_builtins(&self) -> impl Iterator<Item = &str> {
    [Self::HELP, Self::LIST]
      .into_iter()
      .filter(|name| !self.commands.contains_key(*name))
  }

  /// `help`'s lines: every command with how it is written and what it does, then how variables are used.
  fn describe(&self) -> Vec<String> {
    let builtins = self.list_builtins().map(|name| {
      if name == Self::HELP {
        format!("{} - this list", Self::HELP)
      } else {
        format!("{} [part] - the variables, those whose name holds part", Self::LIST)
      }
    });
    let mut lines: Vec<String> = builtins.collect();

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
