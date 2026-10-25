use crate::commands::gamemtl;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `gamemtl` domain and the operations it exposes.
pub(crate) struct GameMtlGroup;

impl GameMtlGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "gamemtl",
      label: "Game materials",
      about: "Game material library tools",
      commands: vec![gamemtl::info::InfoCommand::new_box()],
    }
  }
}
