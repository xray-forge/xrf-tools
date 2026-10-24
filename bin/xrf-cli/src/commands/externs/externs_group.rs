use crate::commands::externs;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `externs` domain and the operations it exposes.
pub(crate) struct ExternsGroup;

impl ExternsGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "externs",
      label: "Externs",
      about: "Script extern export tools",
      commands: vec![externs::export::ExportCommand::new_box()],
    }
  }
}
