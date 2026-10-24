use clap::Command;

use crate::core::command::GenericCommand;

/// Named set of related commands; drives CLI registration and generated documentation. Each domain declares its own in
/// `commands/<domain>/<domain>_group.rs`.
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
    self.commands.iter().fold(
      Command::new(self.slug).about(self.about).arg_required_else_help(true),
      |group, command| group.subcommand(command.init()),
    )
  }
}
