use clap::{ArgMatches, Command};

use crate::core::command::{CommandContext, CommandInput, CommandResult};

/// One CLI operation.
pub(crate) trait GenericCommand: Sync {
  /// A boxed command, as the domain groups register it.
  fn new_box() -> Box<Self>
  where
    Self: Sized + Default,
  {
    Box::new(Self::default())
  }

  /// Operation token within this command's registered domain.
  fn operation(&self) -> &'static str;

  /// What the command reads, declared as its positional argument before [`Self::declare`] adds the rest; a command
  /// reading no path says [`CommandInput::None`], so none leaves the choice unmade.
  fn input(&self) -> CommandInput;

  /// Declares the command's about text and named arguments on `command`, which already carries its name and input.
  fn declare(&self, command: Command) -> Command;

  /// The command as the parser takes it, named for its operation.
  fn init(&self) -> Command {
    self.declare(self.input().declare(Command::new(self.operation())))
  }

  /// Runs the command.
  ///
  /// Reporting is not a command's concern: the reporting flags are declared once for the whole CLI and resolved before
  /// dispatch, and the envelope is written afterwards by the same place that turns this result into an exit code. A
  /// command reads its own arguments, says what it is doing through `context.get_output()`, and deposits any structured
  /// result with [`CommandContext::set_result`].
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult;
}
