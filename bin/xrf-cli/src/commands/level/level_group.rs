use crate::commands::level;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `level` domain and the operations it exposes.
pub(crate) struct LevelGroup;

impl LevelGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "level",
      label: "Level",
      about: "Compiled level tools",
      commands: vec![
        level::albedo::AlbedoCommand::new_box(),
        level::info::InfoCommand::new_box(),
        level::verify::VerifyCommand::new_box(),
      ],
    }
  }
}
