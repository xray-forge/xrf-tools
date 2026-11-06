use crate::commands::object;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `object` domain, the SDK's editable objects, and the operations it exposes.
pub(crate) struct ObjectGroup;

impl ObjectGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "object",
      label: "Object",
      about: "SDK editable object (.object) tools",
      commands: vec![object::decompile::DecompileCommand::new_box()],
    }
  }
}
