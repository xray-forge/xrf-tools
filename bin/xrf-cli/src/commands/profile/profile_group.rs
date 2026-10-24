use crate::commands::profile;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `profile` domain and the operations it exposes.
pub(crate) struct ProfileGroup;

impl ProfileGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "profile",
      label: "Profile",
      about: "Performance measurement tools",
      commands: vec![profile::run::RunCommand::new_box()],
    }
  }
}
