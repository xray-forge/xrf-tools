use crate::commands::gamedata;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `gamedata` domain and the operations it exposes.
pub(crate) struct GamedataGroup;

impl GamedataGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "gamedata",
      label: "Gamedata",
      about: "Assembled gamedata tools",
      commands: vec![
        gamedata::list::ListCommand::new_box(),
        gamedata::verify::VerifyCommand::new_box(),
      ],
    }
  }
}
