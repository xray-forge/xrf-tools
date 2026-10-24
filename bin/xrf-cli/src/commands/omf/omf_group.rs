use crate::commands::omf;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `omf` domain and the operations it exposes.
pub(crate) struct OmfGroup;

impl OmfGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "omf",
      label: "OMF",
      about: "OMF motion tools",
      commands: vec![
        omf::duplicate_motion::DuplicateMotionCommand::new_box(),
        omf::filter_motions::FilterMotionsCommand::new_box(),
        omf::info::InfoCommand::new_box(),
        omf::rename_motions::RenameMotionsCommand::new_box(),
        omf::repack::RepackCommand::new_box(),
      ],
    }
  }
}
