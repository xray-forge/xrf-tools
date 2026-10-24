use xrf_spawn::SpawnFile;

use crate::commands::{packed_file, spawn};
use crate::core::command::{CommandGroup, GenericCommand};

/// The `spawn` domain and the operations it exposes.
pub(crate) struct SpawnGroup;

impl SpawnGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "spawn",
      label: "Spawn",
      about: "Spawn file tools",
      commands: vec![
        spawn::info::InfoCommand::new_box(),
        packed_file::PackCommand::<SpawnFile>::new_box(),
        packed_file::RepackCommand::<SpawnFile>::new_box(),
        packed_file::ReUnpackCommand::<SpawnFile>::new_box(),
        packed_file::UnpackCommand::<SpawnFile>::new_box(),
        packed_file::VerifyCommand::<SpawnFile>::new_box(),
      ],
    }
  }
}
