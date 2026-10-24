use crate::commands::ltx;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `ltx` domain and the operations it exposes.
pub(crate) struct LtxGroup;

impl LtxGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "ltx",
      label: "LTX",
      about: "LTX configuration tools",
      commands: vec![
        ltx::format::FormatCommand::new_box(),
        ltx::inspect::InspectCommand::new_box(),
        ltx::list::ListCommand::new_box(),
        ltx::verify::VerifyCommand::new_box(),
      ],
    }
  }
}
