use crate::commands::thm;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `thm` domain and the operations it exposes.
pub(crate) struct ThmGroup;

impl ThmGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "thm",
      label: "THM",
      about: "THM texture metadata tools",
      commands: vec![
        thm::bump_alignment::BumpAlignmentCommand::new_box(),
        thm::patch_bump::PatchBumpCommand::new_box(),
      ],
    }
  }
}
