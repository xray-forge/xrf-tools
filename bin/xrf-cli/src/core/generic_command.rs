use clap::{ArgMatches, Command};

use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;

pub type CommandResult<T = ()> = Result<T, CommandError>;

/// Named set of related commands; drives CLI registration and generated documentation.
pub struct CommandGroup {
  /// Lowercase command token used by the parser and command paths.
  pub slug: &'static str,
  /// Human-facing domain label, preserving established acronym capitalization.
  pub label: &'static str,
  /// Concise description shown in root CLI help.
  pub about: &'static str,
  pub commands: Vec<Box<dyn GenericCommand>>,
}

impl CommandGroup {
  /// Builds the domain node that owns this group's operation subcommands.
  pub fn init(&self) -> Command {
    let mut group: Command = Command::new(self.slug).about(self.about).arg_required_else_help(true);

    for command in &self.commands {
      group = group.subcommand(command.init());
    }

    group
  }
}

/// One CLI operation.
pub trait GenericCommand: Sync {
  fn new() -> Self
  where
    Self: Sized + Default,
  {
    Self::default()
  }

  fn new_box() -> Box<Self>
  where
    Self: Sized + Default,
  {
    Box::new(Self::default())
  }

  /// Operation token within this command's registered domain.
  fn operation(&self) -> &'static str;

  fn init(&self) -> Command;

  /// Runs the command.
  ///
  /// Reporting is not a command's concern: the reporting flags are declared once for the whole CLI
  /// and resolved before dispatch, and the envelope is written afterwards by the same place that
  /// turns this result into an exit code. A command reads its own arguments, says what it is doing
  /// through `context.get_output()`, and deposits any structured result with
  /// [`CommandContext::set_result`].
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult;
}
