use super::command_reference::CommandReference;
use crate::core::arguments::ReportingArguments;
use crate::core::command::CommandGroup;

/// Documentation model extracted from one registered CLI command group.
pub struct GroupReference {
  pub slug: &'static str,
  pub label: &'static str,
  pub commands: Vec<CommandReference>,
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
        .map(|command| CommandReference::from_command(group.slug, ReportingArguments::declare(command.init())))
        .collect(),
    }
  }
}
