use crate::commands::docs;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `docs` domain and the operations it exposes.
pub(crate) struct DocsGroup;

impl DocsGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "docs",
      label: "Docs",
      about: "CLI documentation tools",
      commands: vec![docs::generate::GenerateCommand::new_box()],
    }
  }
}
