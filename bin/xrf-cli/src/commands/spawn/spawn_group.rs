use xrf_spawn::AnySpawnFile;

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
        packed_file::PackCommand::<AnySpawnFile>::new_box(),
        packed_file::RepackCommand::<AnySpawnFile>::new_box(),
        packed_file::ReUnpackCommand::<AnySpawnFile>::new_box(),
        packed_file::UnpackCommand::<AnySpawnFile>::new_box(),
        packed_file::VerifyCommand::<AnySpawnFile>::new_box(),
      ],
    }
  }
}
