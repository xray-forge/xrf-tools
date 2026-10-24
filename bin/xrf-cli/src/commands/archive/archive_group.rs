use crate::commands::archive;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `archive` domain and the operations it exposes.
pub(crate) struct ArchiveGroup;

impl ArchiveGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "archive",
      label: "Archive",
      about: "X-Ray archive tools",
      commands: vec![
        archive::extract::ExtractCommand::new_box(),
        archive::find::FindCommand::new_box(),
        archive::info::InfoCommand::new_box(),
        archive::list::ListCommand::new_box(),
        archive::pack::PackCommand::new_box(),
        archive::pack_patch::PackPatchCommand::new_box(),
        archive::unpack::UnpackCommand::new_box(),
        archive::verify::VerifyCommand::new_box(),
      ],
    }
  }
}
