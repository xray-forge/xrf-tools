use crate::commands::dialog;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `dialog` domain and the operations it exposes.
pub(crate) struct DialogGroup;

impl DialogGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "dialog",
      label: "Dialog",
      about: "Dialog inspection tools",
      commands: vec![
        dialog::find::FindCommand::new_box(),
        dialog::info::InfoCommand::new_box(),
        dialog::inspect::InspectCommand::new_box(),
        dialog::list::ListCommand::new_box(),
      ],
    }
  }
}
